#!/usr/bin/env python3
"""
20_analyze.py - Phase 2: static analysis of cJSON.c.

Produces:
  artifacts/analysis/callgraph.json    call graph + data dependencies
  artifacts/analysis/macros.json       every macro, classified
  artifacts/analysis/structs.json      struct layouts + globals
  artifacts/analysis/schedule.json     translation units, leaf-first order, context bundles
  artifacts/analysis/unresolved.json   what the analysis could NOT determine

Method:
  * function definitions, signatures, globals and struct layouts come from the
    clang AST (clang -Xclang -ast-dump=json) - not from regex guessing
  * the call graph is built from CallExpr nodes, resolving the callee through
    the AST's referencedDecl ids
  * cycles are found with Tarjan's SCC algorithm; each SCC is translated
    together, units are ordered leaf-first over the condensation DAG
  * the partition into semantic areas is hand-seeded but *machine-verified*
    against the call graph: every edge must point from a later unit to an
    earlier unit, otherwise the violation is reported
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from collections import defaultdict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CJSON_DIR = os.path.join(ROOT, "vendor", "cJSON")
CJSON_C = os.path.join(CJSON_DIR, "cJSON.c")
CJSON_H = os.path.join(CJSON_DIR, "cJSON.h")
OUT_DIR = os.path.join(ROOT, "artifacts", "analysis")
AST = "/tmp/opencode/cjson_ast.json"


# --------------------------------------------------------------------------- #
# AST helpers
# --------------------------------------------------------------------------- #
def load_ast() -> dict:
    if not os.path.isfile(AST) or os.path.getsize(AST) < 1000:
        print("[*] regenerating clang AST dump ...", flush=True)
        with open(AST, "w") as fh:
            subprocess.run(["clang", "-Xclang", "-ast-dump=json", "-fsyntax-only",
                            "-I", CJSON_DIR, CJSON_C],
                           cwd=CJSON_DIR, stdout=fh, stderr=subprocess.DEVNULL, check=True)
    with open(AST) as fh:
        return json.load(fh)


def walk(node, out: list) -> None:
    """Append every dict node in the AST (and its subtree) to `out`."""
    if isinstance(node, list):
        for x in node:
            walk(x, out)
    elif isinstance(node, dict):
        out.append(node)
        for c in node.get("inner") or []:
            walk(c, out)


MAIN_FILE = "cJSON.c"


def node_file(n: dict) -> str:
    """File a node belongs to.

    clang omits `loc.file` for nodes in the main translation unit, so an empty
    result means "the main file" (cJSON.c) rather than "unknown".
    """
    loc = n.get("loc") or {}
    f = loc.get("file")
    if not f:
        # a node pulled in from a header carries the header in `includedFrom`
        f = (loc.get("includedFrom") or {}).get("file")
    return f or MAIN_FILE


# --------------------------------------------------------------------------- #
# 1. functions, signatures, globals, structs
# --------------------------------------------------------------------------- #
def extract(ast: dict):
    nodes = []
    walk(ast.get("inner", []), nodes)

    # id -> name for every FunctionDecl (so CallExpr targets can be resolved)
    fname: dict[str, str] = {}
    for n in nodes:
        if n.get("kind") == "FunctionDecl" and n.get("id"):
            fname[n["id"]] = n.get("name") or ""

    funcs: dict[str, dict] = {}
    for n in nodes:
        if n.get("kind") != "FunctionDecl":
            continue
        body = [c for c in (n.get("inner") or [])
                if isinstance(c, dict) and c.get("kind") == "CompoundStmt"]
        if not body:
            continue
        if node_file(n) != MAIN_FILE:
            continue
        name = n.get("name")
        params = []
        for c in n.get("inner") or []:
            if isinstance(c, dict) and c.get("kind") == "ParmVarDecl":
                params.append({
                    "name": c.get("name") or "",
                    "type": (c.get("type") or {}).get("qualType", ""),
                })
        funcs[name] = {
            "name": name,
            "line": (n.get("loc") or {}).get("line"),
            "return_type": (n.get("type") or {}).get("qualType", ""),
            "params": params,
            "id": n.get("id"),
            "body": body[0],
        }

    # ---- structs and globals ----
    # The clang JSON dump does not reliably attribute header-defined types to
    # their file (loc.file is omitted, includedFrom points at the *including*
    # file), so structs and globals are extracted from the source text instead,
    # where the answer is unambiguous. The AST is still used for the function
    # set, signatures and call graph, where it is exact.
    structs = extract_structs_from_source()
    globals_ = extract_globals_from_source()
    return funcs, globals_, structs, fname


def build_callgraph(funcs: dict, fname: dict) -> dict[str, set[str]]:
    """For each cJSON.c function, the set of cJSON.c functions it calls."""
    graph: dict[str, set[str]] = {k: set() for k in funcs}

    def callee_of(call: dict) -> str | None:
        # a CallExpr's callee is its first child; it may be wrapped in casts
        for child in call.get("inner") or []:
            if not isinstance(child, dict):
                continue
            k = child.get("kind")
            if k == "DeclRefExpr":
                ref = child.get("referencedDecl") or {}
                return ref.get("name")
            if k == "FunctionDecl":
                return child.get("name")
            if k in ("ImplicitCastExpr", "CStyleCastExpr", "ParenExpr"):
                got = callee_of(child)
                if got:
                    return got
        return None

    for name, f in funcs.items():
        calls = set()
        nodes = []
        walk(f["body"], nodes)
        for n in nodes:
            if n.get("kind") == "CallExpr":
                c = callee_of(n)
                if c and c in funcs and c != name:
                    calls.add(c)
        graph[name] = calls
    return graph


# --------------------------------------------------------------------------- #
# 1b. structs and globals, straight from the source text
# --------------------------------------------------------------------------- #
def _strip_comments(src: str) -> str:
    """Remove comments but preserve every byte offset and line number.

    Comment characters become spaces and newlines inside a block comment are
    kept, so a match index in the stripped text maps 1:1 onto the original.
    """
    def blank(m: re.Match) -> str:
        return re.sub(r"[^\n]", " ", m.group(0))

    src = re.sub(r"/\*.*?\*/", blank, src, flags=re.S)
    src = re.sub(r"//[^\n]*", lambda m: " " * len(m.group(0)), src)
    return src


def _parse_fields(body: str) -> list[dict]:
    """Parse the declarations inside a struct body."""
    fields = []
    for decl in body.split(";"):
        d = " ".join(decl.split())
        if not d or d in ("struct", "union", "enum"):
            continue
        # function pointer:  void *(*malloc_fn)(size_t)  /  void *(CJSON_CDECL *allocate)(size_t)
        fm = re.match(r"^(.*?)\(\s*(?:[A-Za-z_][A-Za-z0-9_]*\s+)?\*\s*"
                      r"([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*\((.*)\)$", d)
        if fm:
            fields.append({
                "name": fm.group(2),
                "type": f"{fm.group(1).strip()} (*)({fm.group(3).strip()})",
            })
            continue
        # ordinary field:  const unsigned char *content  /  int type
        fm = re.match(r"^(.*?)([A-Za-z_][A-Za-z0-9_]*)\s*(\[[^\]]*\])?$", d)
        if not fm:
            continue
        name = fm.group(2)
        if name in ("struct", "union", "enum"):
            continue
        # split the declaration into "type part" and the trailing identifier
        head = d[: d.rindex(name)].strip()
        if not head:
            continue
        fields.append({"name": name, "type": head})
    return fields


def extract_structs_from_source() -> dict[str, dict]:
    """Find `typedef struct [tag] { ... } name;` and `struct tag { ... };`."""
    out: dict[str, dict] = {}
    for path in (CJSON_C, CJSON_H):
        raw = open(path, encoding="utf-8", errors="replace").read()
        src = _strip_comments(raw)
        base = os.path.basename(path)
        for m in re.finditer(r"typedef\s+struct\s*([A-Za-z_][A-Za-z0-9_]*)?\s*\{", src):
            i = src.index("{", m.end() - 1)
            depth = 0
            j = i
            while j < len(src):
                if src[j] == "{":
                    depth += 1
                elif src[j] == "}":
                    depth -= 1
                    if depth == 0:
                        break
                j += 1
            body = src[i + 1:j]
            tail = src[j + 1:j + 200]
            tm = re.match(r"\s*([A-Za-z_][A-Za-z0-9_]*)\s*;", tail)
            name = m.group(1) or (tm.group(1) if tm else None)
            if not name:
                continue
            line = raw[:m.start()].count("\n") + 1
            out[name] = {"name": name, "file": base, "line": line,
                         "fields": _parse_fields(body)}
    return out


def extract_globals_from_source() -> list[dict]:
    """File-scope variable definitions (those with an initializer)."""
    out = []
    for path in (CJSON_C, CJSON_H):
        raw = open(path, encoding="utf-8", errors="replace").read()
        src = _strip_comments(raw)
        for m in re.finditer(r"^(static\s+)?(const\s+)?([A-Za-z_][A-Za-z0-9_ \t\*]*?)\s+"
                             r"([A-Za-z_][A-Za-z0-9_]*)\s*=\s*[^;]*;",
                             src, re.M):
            vtype = (m.group(2) or "") + m.group(3)
            vtype = re.sub(r"\s+", " ", vtype).strip()
            if vtype.endswith("*") or "*" in vtype or vtype in ("error", "internal_hooks"):
                out.append({
                    "name": m.group(4),
                    "type": vtype,
                    "line": raw[:m.start()].count("\n") + 1,
                    "is_static": bool(m.group(1)),
                    "file": os.path.basename(path),
                })
    return out


# --------------------------------------------------------------------------- #
# 2. macros
# --------------------------------------------------------------------------- #
def collect_macros() -> list[dict]:
    """All macros visible while compiling cJSON.c, with their definitions."""
    p = subprocess.run(["clang", "-E", "-dD", "-I", CJSON_DIR, CJSON_C],
                       cwd=CJSON_DIR, capture_output=True, text=True)
    src = open(CJSON_C, encoding="utf-8", errors="replace").read()
    hdr = open(CJSON_H, encoding="utf-8", errors="replace").read()
    local_src = set(re.findall(r"^\s*#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)", src, re.M))
    local_hdr = set(re.findall(r"^\s*#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)", hdr, re.M))

    defs: dict[str, str] = {}
    for line in p.stdout.splitlines():
        m = re.match(r"^#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)(\()?", line)
        if m:
            defs.setdefault(m.group(1), line)

    out = []
    for name, line in sorted(defs.items()):
        func_like = bool(re.match(r"^#\s*define\s+[A-Za-z_][A-Za-z0-9_]*\(", line))
        body = line.split("#define", 1)[-1]
        body = re.sub(r"^\s*[A-Za-z_][A-Za-z0-9_]*", "", body, count=1)
        if func_like:
            body = re.sub(r"^\(", "", body, count=1)
        origin = "cJSON.c" if name in local_src else ("cJSON.h" if name in local_hdr else "system/other")
        out.append({
            "name": name,
            "function_like": func_like,
            "origin": origin,
            "definition": line.strip()[:400],
            "body": body.strip()[:200],
        })
    return out


def classify_macro(m: dict) -> str:
    n, b, o = m["name"], m["body"], m["origin"]
    if o in ("cJSON.c", "cJSON.h"):
        return "local"
    if n.startswith("__") or n.startswith("_"):
        return "system"
    if n in ("NULL", "true", "false", "stdin", "stdout", "stderr"):
        return "system-constant"
    if re.fullmatch(r"[A-Z_][A-Z0-9_]*", n):
        return "system-constant"
    return "system"


# --------------------------------------------------------------------------- #
# 2b. macro expansion + the Rust strategy for every LOCAL macro
# --------------------------------------------------------------------------- #
# For every macro defined in cJSON.c / cJSON.h we record:
#   expansion : what the preprocessor actually produces
#   strategy  : how it should be represented in Rust
#   rust_note : the concrete guidance handed to the translator
MACRO_STRATEGY: dict[str, tuple[str, str]] = {
    "CJSON_PUBLIC": ("drop", "C linkage decoration; has no Rust equivalent. Rust items are "
                             "exported with `pub` and the C ABI is added separately."),
    "CJSON_CDECL": ("drop", "calling-convention decoration; drop in Rust."),
    "CJSON_STDCALL": ("drop", "calling-convention decoration; drop in Rust."),
    "CJSON_VERSION_MAJOR": ("inline-const", "pub const CJSON_VERSION_MAJOR: u32 = 1;"),
    "CJSON_VERSION_MINOR": ("inline-const", "pub const CJSON_VERSION_MINOR: u32 = 7;"),
    "CJSON_VERSION_PATCH": ("inline-const", "pub const CJSON_VERSION_PATCH: u32 = 19;"),
    "CJSON_NESTING_LIMIT": ("inline-const",
                             "pub const CJSON_NESTING_LIMIT: usize = 1000;  "
                             "// parse recursion depth limit - MUST be preserved exactly"),
    "CJSON_CIRCULAR_LIMIT": ("inline-const",
                             "pub const CJSON_CIRCULAR_LIMIT: usize = 10000;  "
                             "// duplicate/compare cycle limit - MUST be preserved exactly"),
    "cJSON_Invalid": ("inline-const", "const CJSON_INVALID: u8 = 0;"),
    "cJSON_False": ("inline-const", "const CJSON_FALSE: u8 = 1 << 0;"),
    "cJSON_True": ("inline-const", "const CJSON_TRUE: u8 = 1 << 1;"),
    "cJSON_NULL": ("inline-const", "const CJSON_NULL: u8 = 1 << 2;"),
    "cJSON_Number": ("inline-const", "const CJSON_NUMBER: u8 = 1 << 3;"),
    "cJSON_String": ("inline-const", "const CJSON_STRING: u8 = 1 << 4;"),
    "cJSON_Array": ("inline-const", "const CJSON_ARRAY: u8 = 1 << 5;"),
    "cJSON_Object": ("inline-const", "const CJSON_OBJECT: u8 = 1 << 6;"),
    "cJSON_Raw": ("inline-const", "const CJSON_RAW: u8 = 1 << 7;"),
    "cJSON_IsReference": ("inline-const", "const CJSON_IS_REFERENCE: u8 = 256;"),
    "cJSON_StringIsConst": ("inline-const", "const CJSON_STRING_IS_CONST: u8 = 512;"),
    "cJSON_ArrayForEach": ("rust-helper",
                            "becomes an iterator: `node.children()` returning "
                            "`impl Iterator<Item = &Node>` - do NOT port the for-loop macro"),
    "cJSON_SetBoolValue": ("rust-helper",
                           "becomes a method `Node::set_bool(&mut self, v: bool)`"),
    "cJSON_SetIntValue": ("rust-helper",
                          "becomes `Node::set_number(&mut self, v: f64)` (C truncates to int)"),
    "cJSON_SetNumberValue": ("rust-helper",
                              "becomes `Node::set_number(&mut self, v: f64)`"),
    "buffer_at_offset": ("inline-fn",
                         "inline as `&buffer.content[buffer.offset..]`; the C version can run "
                         "past the end - the Rust version must not"),
    "can_access_at_index": ("rust-helper",
                            "becomes `buffer.can_access_at_index(i) -> bool` with a checked "
                            "bounds test (this is a Kani proof target)"),
    "cannot_access_at_index": ("rust-helper",
                                "becomes `!buffer.can_access_at_index(i)`"),
    "can_read": ("rust-helper",
                 "becomes `buffer.can_read(n) -> bool` with a checked bounds test "
                 "(this is a Kani proof target)"),
    "cjson_min": ("inline-fn", "inline as Rust `a.min(b)`"),
    "static_strlen": ("inline-fn",
                      "C's `sizeof(literal) - 1`; in Rust use `literal.len()` on a `&str`"),
    "true": ("inline-const", "Rust's `true`"),
    "false": ("inline-const", "Rust's `false`"),
    "NAN": ("inline-fn", "Rust's `f64::NAN`"),
    "isnan": ("inline-fn", "Rust's `f64::is_nan()`"),
    "isinf": ("inline-fn", "Rust's `f64::is_infinite()`"),
    "internal_malloc": ("inline-fn",
                        "becomes `Vec::with_capacity`/allocator call; the Rust code must own "
                        "its allocations, so do NOT reproduce a raw malloc wrapper"),
    "internal_free": ("inline-fn", "dropped - Rust frees automatically via `Drop`"),
    "internal_realloc": ("inline-fn",
                         "becomes `Vec::reserve` / `Vec::resize` - Rust grows without realloc"),
    "cJSON__h": ("drop", "header guard; drop."),
}


def expand_macro(name: str, body: str, function_like: bool) -> str:
    """Ask the preprocessor what a macro expands to (object-like macros only).

    A unique marker is planted after the include so the expansion can be
    isolated from the rest of the header's preprocessed output.
    """
    if function_like:
        return ""
    marker = f"__MACRO_PROBE_{name}__"
    probe = f'#include "cJSON.h"\n{marker} {body}\n'
    p = subprocess.run(["clang", "-E", "-P", "-x", "c", "-", "-I", CJSON_DIR],
                       input=probe, capture_output=True, text=True)
    for line in (p.stdout or "").splitlines():
        if marker in line:
            return line.replace(marker, "").strip()
    return ""


def enrich_macros(macros: list[dict]) -> None:
    for m in macros:
        strat, note = MACRO_STRATEGY.get(m["name"], ("inline-fn", "expand inline"))
        m["strategy"] = strat
        m["rust_note"] = note
        m["expansion"] = expand_macro(m["name"], m["body"], m["function_like"]) \
            if m["origin"] in ("cJSON.c", "cJSON.h") and not m["function_like"] else ""
        if m["name"] == "cJSON_ArrayForEach":
            m["expansion"] = ("for(element = (array != NULL) ? (array)->child : NULL; "
                              "element != NULL; element = element->next)")
        if m["name"] == "cJSON_SetBoolValue":
            m["expansion"] = ("(object != NULL && ((object)->type & (cJSON_False|cJSON_True))) ? "
                              "((object)->type = ((object)->type & (~(cJSON_False|cJSON_True))) | "
                              "((boolValue) ? cJSON_True : cJSON_False)) : (boolValue) ? "
                              "((object)->type = (object)->type | cJSON_True) : "
                              "((object)->type = (object)->type & (~cJSON_True))")
        if m["name"] == "cJSON_SetIntValue":
            m["expansion"] = ("((object) ? (object)->valueint = (object)->valuedouble = (number) : "
                              "(number))")
        if m["name"] == "cJSON_SetNumberValue":
            m["expansion"] = ("((object != NULL) ? cJSON_SetNumberHelper(object, (double)number) : "
                              "(number))")


# --------------------------------------------------------------------------- #
# 3. strongly connected components (Tarjan)
# --------------------------------------------------------------------------- #
def tarjan_scc(nodes: list[str], graph: dict[str, set[str]]) -> list[list[str]]:
    index: dict[str, int] = {}
    low: dict[str, int] = {}
    on_stack: set[str] = set()
    stack: list[str] = []
    result: list[list[str]] = []
    counter = [0]

    def strongconnect(v: str):
        index[v] = low[v] = counter[0]
        counter[0] += 1
        stack.append(v)
        on_stack.add(v)
        for w in graph.get(v, ()):
            if w not in index:
                strongconnect(w)
                low[v] = min(low[v], low[w])
            elif w in on_stack:
                low[v] = min(low[v], index[w])
        if low[v] == index[v]:
            comp = []
            while True:
                w = stack.pop()
                on_stack.discard(w)
                comp.append(w)
                if w == v:
                    break
            result.append(sorted(comp))

    for n in nodes:
        if n not in index:
            strongconnect(n)
    return result


# --------------------------------------------------------------------------- #
# 4. semantic areas (hand-seeded, machine-verified against the call graph)
# --------------------------------------------------------------------------- #
AREAS: list[tuple[str, str, list[str]]] = [
    ("U01_hooks_alloc",
     "allocator hooks, strdup, node lifecycle",
     ["cJSON_InitHooks", "cJSON_strdup",
      "cJSON_New_Item", "cJSON_Delete", "cJSON_malloc", "cJSON_free"]),
    ("U02_print_buffer",
     "printbuffer growth and offset bookkeeping",
     ["ensure", "update_offset"]),
    ("U03_number_parse",
     "number scanning and decimal point",
     ["get_decimal_point", "parse_number", "cJSON_SetNumberHelper",
      "compare_double"]),
    ("U04_string_parse",
     "hex4 decoding, UTF-16 -> UTF-8, string parsing",
     ["parse_hex4", "utf16_literal_to_utf8", "parse_string"]),
    ("U05_string_print",
     "string escaping for output",
     ["print_string_ptr", "print_string"]),
    ("U06_value_parse",
     "the mutually recursive value/array/object parsers (one SCC)",
     ["parse_value", "parse_array", "parse_object",
      "buffer_skip_whitespace", "skip_utf8_bom"]),
    ("U06b_parse_entry",
     "the public parse entry points",
     ["cJSON_ParseWithOpts", "cJSON_ParseWithLengthOpts",
      "cJSON_Parse", "cJSON_ParseWithLength"]),
    ("U07_value_print",
     "value/array/object printing and the public print entry points",
     ["print_number", "print_value", "print_array", "print_object", "print",
      "cJSON_Print", "cJSON_PrintUnformatted", "cJSON_PrintBuffered",
      "cJSON_PrintPreallocated"]),
    ("U08_accessors",
     "read-only accessors and type predicates",
     ["cJSON_GetArraySize", "get_array_item", "cJSON_GetArrayItem",
      "get_object_item", "cJSON_GetObjectItem", "cJSON_GetObjectItemCaseSensitive",
      "cJSON_HasObjectItem", "cJSON_GetStringValue", "cJSON_GetNumberValue",
      "cJSON_GetErrorPtr", "cJSON_Version", "case_insensitive_strcmp",
      "cJSON_IsInvalid", "cJSON_IsFalse", "cJSON_IsTrue", "cJSON_IsBool",
      "cJSON_IsNull", "cJSON_IsNumber", "cJSON_IsString", "cJSON_IsArray",
      "cJSON_IsObject", "cJSON_IsRaw"]),
    ("U09_create",
     "node constructors",
     ["cJSON_CreateNull", "cJSON_CreateTrue", "cJSON_CreateFalse",
      "cJSON_CreateBool", "cJSON_CreateNumber", "cJSON_CreateString",
      "cJSON_CreateStringReference", "cJSON_CreateObjectReference",
      "cJSON_CreateArrayReference", "cJSON_CreateRaw", "cJSON_CreateArray",
      "cJSON_CreateObject", "cJSON_CreateIntArray", "cJSON_CreateFloatArray",
      "cJSON_CreateDoubleArray", "cJSON_CreateStringArray",
      "create_reference", "cJSON_SetValuestring",
      "suffix_object", "cast_away_const"]),
    ("U10_add_item",
     "attaching nodes to arrays and objects",
     ["add_item_to_array", "cJSON_AddItemToArray",
      "add_item_to_object", "cJSON_AddItemToObject",
      "cJSON_AddItemToObjectCS", "cJSON_AddItemReferenceToArray",
      "cJSON_AddItemReferenceToObject", "cJSON_AddNullToObject",
      "cJSON_AddTrueToObject", "cJSON_AddFalseToObject",
      "cJSON_AddBoolToObject", "cJSON_AddNumberToObject",
      "cJSON_AddStringToObject", "cJSON_AddRawToObject",
      "cJSON_AddObjectToObject", "cJSON_AddArrayToObject"]),
    ("U11_detach_replace",
     "detaching, deleting and replacing nodes",
     ["cJSON_DetachItemViaPointer", "cJSON_DetachItemFromArray",
      "cJSON_DeleteItemFromArray", "cJSON_DetachItemFromObject",
      "cJSON_DetachItemFromObjectCaseSensitive", "cJSON_DeleteItemFromObject",
      "cJSON_DeleteItemFromObjectCaseSensitive", "cJSON_InsertItemInArray",
      "cJSON_ReplaceItemViaPointer", "cJSON_ReplaceItemInArray",
      "replace_item_in_object", "cJSON_ReplaceItemInObject",
      "cJSON_ReplaceItemInObjectCaseSensitive"]),
    ("U12_duplicate",
     "deep copy",
     ["cJSON_Duplicate", "cJSON_Duplicate_rec"]),
    ("U13_minify",
     "whitespace/comment stripping",
     ["skip_oneline_comment", "skip_multiline_comment", "minify_string",
      "cJSON_Minify"]),
    ("U14_compare",
     "deep structural comparison (used by cJSON_Utils)",
     ["cJSON_Compare"]),
]


# --------------------------------------------------------------------------- #
# 5. pointer role classification
# --------------------------------------------------------------------------- #
OWNING_FUNCS = {
    # functions whose returned pointer is freshly allocated and owned by the caller
    "cJSON_strdup", "print", "cJSON_Print", "cJSON_PrintUnformatted",
    "cJSON_PrintBuffered", "cJSON_Parse", "cJSON_ParseWithOpts",
    "cJSON_ParseWithLength", "cJSON_ParseWithLengthOpts",
    "cJSON_CreateNull", "cJSON_CreateTrue", "cJSON_CreateFalse",
    "cJSON_CreateBool", "cJSON_CreateNumber", "cJSON_CreateString",
    "cJSON_CreateRaw", "cJSON_CreateArray", "cJSON_CreateObject",
    "cJSON_Duplicate", "cJSON_Duplicate_rec", "cJSON_New_Item",
    "cJSON_DetachItemFromArray", "cJSON_DetachItemFromObject",
    "cJSON_DetachItemFromObjectCaseSensitive", "cJSON_DetachItemViaPointer",
    "cJSON_CreateIntArray", "cJSON_CreateFloatArray", "cJSON_CreateDoubleArray",
    "cJSON_CreateStringArray", "cJSON_CreateStringReference",
    "cJSON_CreateObjectReference", "cJSON_CreateArrayReference",
}
# functions that take ownership of the item they are given
TAKES_OWNERSHIP = {"cJSON_AddItemToArray", "cJSON_AddItemToObject",
                    "cJSON_AddItemToObjectCS", "cJSON_ReplaceItemViaPointer",
                    "cJSON_ReplaceItemInArray", "cJSON_ReplaceItemInObject",
                    "cJSON_ReplaceItemInObjectCaseSensitive",
                    "cJSON_InsertItemInArray"}
# functions that free the item they are given
FREES_OWNED = {"cJSON_Delete", "cJSON_DeleteItemFromArray",
               "cJSON_DeleteItemFromObject", "cJSON_DeleteItemFromObjectCaseSensitive"}


def classify_param_role(fname: str, pname: str, ptype: str) -> str:
    t = ptype.replace("const", "").replace("*", "").strip()
    is_ptr = "*" in ptype
    is_const_ptr = "const" in ptype and re.search(r"\*\s*const", ptype) is not None
    is_double_ptr = ptype.count("*") >= 2

    if is_double_ptr:
        return "output-parameter"
    if not is_ptr:
        return "by-value"
    if fname in ("cJSON_ParseWithOpts", "cJSON_ParseWithLengthOpts") and pname == "return_parse_end":
        return "output-parameter"
    if "size_t" in ptype and fname in ("cJSON_ParseWithLengthOpts", "cJSON_ParseWithLength"):
        return "length"
    if fname in ("parse_string", "parse_number", "parse_value", "parse_array",
                 "parse_object", "print_value", "print_array", "print_object",
                 "print_string", "print_number", "print_string_ptr",
                 "buffer_skip_whitespace", "skip_utf8_bom", "ensure",
                 "update_offset"):
        return "borrowed-mutable"
    if "cJSON *" in ptype and "const" not in ptype:
        if fname in FREES_OWNED:
            return "owned-consumed"
        if fname in TAKES_OWNERSHIP:
            return "owned-transferred"
        return "tree-handle"
    if "const" in ptype:
        return "borrowed"
    return "borrowed-mutable"


def classify_return(fname: str, rtype: str) -> str:
    if rtype.strip() == "void":
        return "void"
    if "*" in rtype:
        if fname in OWNING_FUNCS:
            return "owning-pointer"
        if "const" in rtype:
            return "borrowed-pointer"
        return "raw-pointer"
    return "by-value"


# --------------------------------------------------------------------------- #
# main
# --------------------------------------------------------------------------- #
def main() -> None:
    global AREAS
    os.makedirs(OUT_DIR, exist_ok=True)
    print("[*] loading AST ...", flush=True)
    ast = load_ast()
    funcs, globals_, structs, fname_ids = extract(ast)
    print(f"    {len(funcs)} function definitions in cJSON.c", flush=True)

    graph = build_callgraph(funcs, fname_ids)
    print("[*] collecting macros ...", flush=True)
    macros = collect_macros()
    for m in macros:
        m["class"] = classify_macro(m)
    enrich_macros(macros)

    # ---- data dependencies: which functions touch which globals / structs ----
    data_dep: dict[str, dict] = {}
    for name, f in funcs.items():
        g, s = set(), set()
        nodes = []
        walk(f["body"], nodes)
        for n in nodes:
            if n.get("kind") == "DeclRefExpr":
                ref = n.get("referencedDecl") or {}
                if ref.get("kind") == "VarDecl":
                    g.add(ref.get("name"))
                elif ref.get("kind") == "RecordDecl":
                    s.add(ref.get("name"))
        data_dep[name] = {"globals": sorted(x for x in g if x), "types": sorted(x for x in s if x)}

    # ---- SCCs ----
    names = sorted(funcs)
    sccs = tarjan_scc(names, graph)
    scc_of: dict[str, int] = {}
    for i, comp in enumerate(sccs):
        for n in comp:
            scc_of[n] = i
    cyclic = [c for c in sccs if len(c) > 1]
    self_loop = [c[0] for c in sccs if len(c) == 1 and c[0] in graph.get(c[0], ())]

    # ---- partition into areas; verify against the call graph ----
    # ---- split oversized units -------------------------------------------------
    # A 4B model cannot emit 20 correct functions in one answer. Units larger
    # than MAX_UNIT_FUNCTIONS are split into chunks, ordered so that a
    # function's callees inside the same area always land in an EARLIER chunk.
    # Splitting a unit never breaks the leaf-first property of the schedule.
    MAX_UNIT_FUNCTIONS = 6

    def order_within(area: str, members: list[str]) -> list[list[str]]:
        """Order the members of one area into leaf-first *groups*.

        A strongly connected component is atomic: its functions are mutually
        recursive and must be translated together, so a chunk boundary is
        never placed inside one. The groups are then returned in leaf-first
        order (a group's callees come before it).
        """
        member_set = set(members)
        # SCCs restricted to this area
        sccs_local = tarjan_scc(sorted(member_set), graph)
        comp_of = {n: i for i, comp in enumerate(sccs_local) for n in comp}
        # condensation edges: comp(c) -> comp(m) when m calls c
        indeg = {i: 0 for i in range(len(sccs_local))}
        adj: dict[int, list[int]] = {i: [] for i in range(len(sccs_local))}
        for m in members:
            for c in graph.get(m, ()):
                if c in member_set and comp_of[c] != comp_of[m]:
                    if comp_of[m] not in adj[comp_of[c]]:
                        adj[comp_of[c]].append(comp_of[m])
                        indeg[comp_of[m]] += 1
        ready = sorted([i for i in range(len(sccs_local)) if indeg[i] == 0])
        groups: list[list[str]] = []
        while ready:
            i = ready.pop(0)
            groups.append(sccs_local[i])
            for nxt in adj[i]:
                indeg[nxt] -= 1
                if indeg[nxt] == 0:
                    ready.append(nxt)
                    ready.sort()
        # any SCC left over participates in a cycle across SCCs (should not
        # happen, but never drop a function)
        seen = {n for g in groups for n in g}
        for comp in sccs_local:
            if comp[0] not in seen:
                groups.append(comp)
        return groups

    new_areas: list[tuple[str, str, list[str]]] = []
    for area, desc, fs in AREAS:
        members = sorted(set(fs) & set(funcs))
        if not members:
            continue
        if len(members) <= MAX_UNIT_FUNCTIONS:
            new_areas.append((area, desc, members))
            continue
        groups = order_within(area, members)
        # pack the SCC groups into chunks of at most MAX_UNIT_FUNCTIONS
        chunks: list[list[str]] = []
        cur: list[str] = []
        for g in groups:
            if cur and len(cur) + len(g) > MAX_UNIT_FUNCTIONS:
                chunks.append(cur)
                cur = []
            cur.extend(g)
        if cur:
            chunks.append(cur)
        base = re.sub(r"^U(\d+)", r"U\1", area)
        for i, chunk in enumerate(chunks):
            suffix = "" if len(chunks) == 1 else f"_p{i + 1}"
            new_areas.append((base + suffix,
                              desc + (f" (part {i + 1} of {len(chunks)})" if len(chunks) > 1 else ""),
                              chunk))
    AREAS = new_areas

    area_of: dict[str, str] = {}
    dup_seed: dict[str, list[str]] = defaultdict(list)
    for area, _desc, fs in AREAS:
        for f in fs:
            dup_seed[f].append(area)
            area_of.setdefault(f, area)
    seed_conflicts = {f: a for f, a in dup_seed.items() if len(a) > 1 and f in funcs}

    unassigned = sorted(set(funcs) - set(area_of))
    # anything unassigned goes to the last area as a catch-all
    if unassigned:
        fallback = AREAS[-1][0]
        for f in unassigned:
            area_of[f] = fallback

    order = [a[0] for a in AREAS]
    pos = {a: i for i, a in enumerate(order)}
    violations = []
    for caller, callees in graph.items():
        for callee in callees:
            ac, al = area_of[caller], area_of[callee]
            if ac == al:
                continue
            if pos[ac] <= pos[al]:
                violations.append({
                    "caller": caller, "callee": callee,
                    "caller_area": ac, "callee_area": al,
                    "reason": "caller unit is not later than callee unit (leaf-first violated)",
                })

    # ---- context bundles ----
    area_funcs: dict[str, list[str]] = defaultdict(list)
    for f, a in area_of.items():
        area_funcs[a].append(f)

    bundles = []
    for idx, (area, desc, fs) in enumerate(AREAS):
        members = sorted(f for f in fs if f in funcs and area_of.get(f) == area)
        if not members:
            continue
        # external callees = callees living in earlier units
        ext: set[str] = set()
        for m in members:
            for c in graph.get(m, ()):
                if area_of[c] != area:
                    ext.add(c)
        earlier = sorted(c for c in ext if pos[area_of[c]] < pos[area])
        later = sorted(c for c in ext if pos[area_of[c]] > pos[area])
        bundles.append({
            "unit": area,
            "index": idx,
            "description": desc,
            "functions": members,
            "function_count": len(members),
            "lines": [funcs[m]["line"] for m in members],
            "signatures": [{
                "name": m,
                "c_signature": c_signature(funcs[m]),
                "return_role": classify_return(m, funcs[m]["return_type"]),
                "params": [{
                    "name": p["name"],
                    "c_type": p["type"],
                    "role": classify_param_role(m, p["name"], p["type"]),
                } for p in funcs[m]["params"]],
            } for m in members],
            "structs_used": sorted({s for m in members for s in data_dep[m]["types"]}),
            "globals_used": sorted({g for m in members for g in data_dep[m]["globals"]}),
            "calls_earlier_units": earlier,
            "calls_later_units": later,
            "scc_id": sorted({scc_of[m] for m in members}),
            "in_cyclic_scc": any(len(sccs[scc_of[m]]) > 1 for m in members),
        })

    # ---- per-unit macro usage (which macros each unit must expand) ----
    unit_src = extract_unit_sources(funcs)
    macro_by_name = {m["name"]: m for m in macros}
    for b in bundles:
        used = set()
        for m in re.findall(r"\b[A-Za-z_][A-Za-z0-9_]*\b", unit_src.get(b["unit"], "")):
            if m in macro_by_name and macro_by_name[m]["class"] == "local":
                used.add(m)
        b["macros_used"] = sorted(used)
        b["macro_guidance"] = [{
            "name": m,
            "strategy": macro_by_name[m]["strategy"],
            "expansion": macro_by_name[m]["expansion"],
            "rust_note": macro_by_name[m]["rust_note"],
        } for m in sorted(used)]

    # ---- write artifacts ----
    with open(os.path.join(OUT_DIR, "callgraph.json"), "w") as fh:
        json.dump({
            "functions": {k: {kk: vv for kk, vv in v.items() if kk != "body"}
                          for k, v in funcs.items()},
            "edges": {k: sorted(v) for k, v in graph.items()},
            "data_dependencies": data_dep,
            "sccs": sccs,
            "cyclic_sccs": cyclic,
            "self_recursive": self_loop,
        }, fh, indent=2)

    local_macros = [m for m in macros if m["class"] == "local"]
    with open(os.path.join(OUT_DIR, "macros.json"), "w") as fh:
        json.dump({"total": len(macros), "by_class": {
            c: sum(1 for m in macros if m["class"] == c)
            for c in sorted({m["class"] for m in macros})},
            "local_macros": local_macros,
            "macros": macros}, fh, indent=2)

    with open(os.path.join(OUT_DIR, "structs.json"), "w") as fh:
        json.dump({"structs": structs, "globals": globals_}, fh, indent=2)

    with open(os.path.join(OUT_DIR, "schedule.json"), "w") as fh:
        json.dump({
            "target": "cJSON v1.7.19 (c859b25da02955fef659d658b8f324b5cde87be3)",
            "unit_count": len(bundles),
            "order": [b["unit"] for b in bundles],
            "units": bundles,
            "area_of_function": area_of,
            "order_violations": violations,
        }, fh, indent=2)

    # ---- unresolved ----
    unresolved = []
    if violations:
        unresolved.append({
            "kind": "leaf-first-violation",
            "detail": "call edges that go from a later unit to an earlier unit",
            "items": violations,
        })
    if cyclic:
        unresolved.append({
            "kind": "recursive-scc",
            "detail": "these functions are mutually recursive and MUST be translated together",
            "items": cyclic,
        })
    if self_loop:
        unresolved.append({
            "kind": "self-recursive",
            "detail": "direct self-recursion",
            "items": self_loop,
        })
    if seed_conflicts:
        unresolved.append({
            "kind": "partition-conflict",
            "detail": "functions seeded into more than one area; the first area in AREAS wins",
            "items": seed_conflicts,
        })
    unresolved.append({
        "kind": "analysis-limitation",
        "detail": "pointer roles are heuristic (syntax + known-function table); "
                  "the aliasing/ownership semantics of `cJSON` trees are NOT proven "
                  "by static analysis - they are established empirically in Phase 4",
        "items": [],
    })
    unresolved.append({
        "kind": "analysis-limitation",
        "detail": "the call graph contains only direct calls; indirect calls through "
                  "function pointers (hooks, localeconv) are not resolved",
        "items": sorted({f for f, g in graph.items()
                         if any("hooks" in str(funcs[f]["params"]) for _ in [0])}),
    })
    with open(os.path.join(OUT_DIR, "unresolved.json"), "w") as fh:
        json.dump(unresolved, fh, indent=2)

    # ---- summary ----
    print()
    print(f"functions           : {len(funcs)}")
    print(f"call edges          : {sum(len(v) for v in graph.values())}")
    print(f"cyclic SCCs         : {len(cyclic)} {cyclic}")
    print(f"self-recursive      : {len(self_loop)} {self_loop}")
    print(f"macros              : {len(macros)}")
    print(f"units               : {len(bundles)}")
    print(f"leaf-first violations: {len(violations)}")
    for v in violations[:10]:
        print(f"    {v['caller']} ({v['caller_area']}) -> {v['callee']} ({v['callee_area']})")
    print()
    print("schedule (leaf-first):")
    for b in bundles:
        cyc = " [cyclic]" if b["in_cyclic_scc"] else ""
        print(f"  {b['index']:2d} {b['unit']:<24} {b['function_count']:>2} fns{cyc}")


def c_signature(f: dict) -> str:
    ps = ", ".join(f"{p['type']} {p['name']}".strip() for p in f["params"])
    return f"{f['return_type']} {f['name']}({ps})"


def extract_function_text(name: str) -> str:
    """The C source text of one function definition (brace matched)."""
    raw = open(CJSON_C, encoding="utf-8", errors="replace").read()
    src = _strip_comments(raw)
    pattern = re.compile(r"\b" + re.escape(name) + r"\s*\(")
    for m in pattern.finditer(src):
        i = src.index("(", m.start())
        depth, j = 0, i
        while j < len(src):
            if src[j] == "(":
                depth += 1
            elif src[j] == ")":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        k = j + 1
        while k < len(src) and src[k] in " \t\r\n":
            k += 1
        if k >= len(src) or src[k] != "{":
            continue
        start = src.rfind("\n", 0, m.start())
        start = 0 if start == -1 else start + 1
        depth = 0
        for e in range(k, len(src)):
            if src[e] == "{":
                depth += 1
            elif src[e] == "}":
                depth -= 1
                if depth == 0:
                    return raw[start:e + 1]
    return ""


def extract_unit_sources(funcs: dict) -> dict[str, str]:
    """Concatenated C source of every function in each unit."""
    area_of: dict[str, str] = {}
    for area, _d, fs in AREAS:
        for f in fs:
            area_of.setdefault(f, area)
    out: dict[str, str] = {}
    for area, _d, fs in AREAS:
        parts = []
        for f in sorted(set(fs) & set(funcs)):
            parts.append(extract_function_text(f))
        out[area] = "\n\n".join(p for p in parts if p)
    return out


if __name__ == "__main__":
    main()
