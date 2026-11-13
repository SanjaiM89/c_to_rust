#!/usr/bin/env python3
"""
30_translate.py - Phase 3: the idiomatic translation pass.

For every unit in the leaf-first schedule we:
  1. build a prompt from the unit's C source + the Rust contract + the target
     signatures + the Rust signatures of the units already translated
  2. ask the model for ONE Rust code block
  3. save the candidate (never overwriting an earlier candidate)
  4. assemble the crate so far (contract + units 0..i) and run `cargo check`
     to measure the compile-first-try rate

Two arms are supported, so the "same idiomatic pass with the local 3B model"
comparison the brief asks for is a single code path:
  --model <model-gguf-name> --out <dir>

Usage:
  30_translate.py --model Qwen3-4B-Instruct-2507-Q4_K_M --out artifacts/final
  30_translate.py --model qwen2.5-coder-3b-instruct-q4_k_m --out artifacts/arm_qwen25
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import contract as contract_mod  # noqa: E402
from llm_client import LLMConfig, ask  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
A = os.path.join(ROOT, "artifacts", "analysis")

SYSTEM = """You are a senior systems engineer translating C to Rust.
You write SAFE, IDIOMATIC Rust. Your reply is compiled, so it must be exact.

CHECKLIST - every line matters:
1. Reply with EXACTLY ONE ```rust code block. No prose.
2. NO `unsafe`. NO raw pointers. NO `libc`, `CString`, `mem::transmute`, `std::ptr`.
   A C `char *` that walks a buffer becomes a `usize` index, NOT a Rust pointer.
3. NO C pointer idioms: no `is_null()`, no `null_mut()`, no `as *mut`, no `NULL`.
   Rust has no null pointers. `Option` is `None`. A `Vec` is never null.
   `is_none()` is ONLY valid on an `Option`. A `&Node`/`&mut Node`/`&[u8]`
   parameter is never null, so its NULL check disappears.
4. NO C hook/allocator plumbing: the C `hooks` / `internal_hooks` allocator calls
   have no counterpart. `Vec` grows by itself. Delete that code.
5. Use ONLY the fields, methods and functions that appear in the contract and
   in the given signatures. Do not invent fields.
6. Implement EXACTLY the given signatures. Same names, same order, same types.
7. Every function is `pub fn`.
8. JSON payload data is RAW BYTES: `&[u8]` in, `Vec<u8>` out. Never `&str`/`String`.
9. The C `child`/`next`/`prev` chain is `children: Vec<Node>`. There is NO
   `next` field, NO `prev` field and NO `child` field on `Node`.
   Iterate with `node.children.iter()`.
10. Preserve the C behaviour exactly, including every error/edge case.
11. Output only the functions you were asked for. No `mod`, no `use`, no `impl`,
    no `fn main`, no tests, no type definitions."""


WORKED_EXAMPLES = """## Worked examples (follow these patterns exactly)

### A. the C sibling loop becomes a Vec iteration
```c
static void print_children(const cJSON *item, printbuffer *p) {
    const cJSON *child;
    for (child = item->child; child != NULL; child = child->next) {
        print_value(child, p);
    }
}
```
```rust
pub fn print_children(item: &Node, p: &mut PrintBuffer) {
    for child in item.children.iter() {
        let _ = print_value(child, p);
    }
}
```
There is NO `next` field and NO `prev` field on `Node`. The children of a node
live in `node.children: Vec<Node>`.

### B. a NULL check on a reference disappears
```c
static cJSON_bool add_item(cJSON *object, cJSON *item) {
    if ((object == NULL) || (item == NULL)) { return false; }
    object->child = item;
    return true;
}
```
```rust
pub fn add_item(object: &mut Node, item: Node) -> bool {
    object.children.push(item);
    return true;
}
```
A `&mut Node` / `&Node` / `&[u8]` parameter can never be null, so its NULL
check simply disappears. `is_none()` is only valid on an `Option`.

### C. the type field is a bitmask
```c
if ((item->type & cJSON_IsReference) == 0) { ... }
```
```rust
if item.type_ & CJSON_IS_REFERENCE == 0 { ... }
```
Integers have no `contains()` method.

### D. bytes, not characters
```c
if ((*input >= 'A') && (*input <= 'Z')) { *output = (unsigned char)((*input) + 32); }
```
```rust
if b'A' <= *input && *input <= b'Z' {
    *output = to_lower(*input);
}
```
A C character literal `'A'` becomes the Rust byte literal `b'A'`.

### E. allocating a copy of a string
```c
copy = (char*)cJSON_strdup(valuestring, &global_hooks);
if (copy == NULL) { return NULL; }
```
```rust
let copy = cjson_strdup(valuestring, &global_hooks())?;
```
`global_hooks()` is a function that returns the installed hooks.

### F. reading a byte and advancing the cursor
```c
if ((buffer->offset + 1) >= buffer->length) { return false; }
c = buffer->content[buffer->offset];
buffer->offset++;
```
```rust
if buffer.offset + 1 >= buffer.length { return false; }
let c = buffer.content[buffer.offset];
buffer.offset += 1;
```

"""
PROMPT = """Translate this C code to safe, idiomatic Rust.

## 1. Rust contract - ALREADY WRITTEN, do not redefine these types
```rust
{contract}
```
The contract is EXHAUSTIVE: these are the only types, fields, constants and
helper functions that exist. Do not use any field, method or function that is
not shown here.

{examples}

## 2. Functions to implement (exact signatures required)
```rust
{signatures}
```

## 3. Rust signatures of functions from other units (already translated, call them directly)
```rust
{callees}
```

## 4. Mechanical C->Rust translation of the same code (faithful but UNSAFE - rewrite it)
```rust
{baseline}
```
(Rewrite it. Do not keep the pointer code.)

## 5. Macro guidance
{macro_notes}

## 6. C source to translate (the behaviour you must preserve)
```c
{csource}
```

## How to convert the C idioms
The Rust types keep the C field names, so most code translates directly:
- `item->type` -> `item.type_` ; `item->valuestring` -> `item.valuestring` ;
  `item->valueint` -> `item.valueint` ; `item->valuedouble` -> `item.valuedouble` ;
  `item->string` -> `item.string`.
- `item->child` -> `item.children.first()` (or `item.child()`).
- `item->next` -> the NEXT element of the `children` Vec. There is no `next` field.
- `item->prev` -> the PREVIOUS element of the `children` Vec.
- `array->child = item` (append) -> `array.children.push(item)`.
- `strcpy(dst, "lit")` -> `dst.copy_from_slice(b"lit")`.
- `strlen(s)` -> `s.len()`.
- `memcpy(d, s, n)` -> `d[..n].copy_from_slice(&s[..n])`.
- `sprintf(buf, "%s", x)` -> build the bytes manually into a `Vec<u8>`.
- `'0'` (a C char literal) -> `b'0'` (a Rust u8). Same for every other literal.
- A C bitmask test `x & FLAG` -> `x & FLAG != 0`. There is no `.contains()` on an integer.
- A C bitmask clear `x &= ~FLAG` -> `x &= !FLAG`.
- The contract's methods need `()`: `item.child()` not `item.child`.
- `INT_MAX` -> `i32::MAX as usize`.
- `NULL` -> `None`; `x == NULL` -> `x.is_none()` ONLY for an `Option`. A `&[u8]`
  parameter is never `None`, so never test it for null.
- A `char *` that the function allocates and returns -> `Option<Vec<u8>>`.
- A `char *` that the function writes into -> `&mut Vec<u8>`.
- `cJSON_Delete(item)` frees memory: in Rust nothing has to be done, the `Node`
  is dropped automatically, so those calls simply disappear.
- The C `hooks` struct fields exist in Rust (`hooks.allocate`, `hooks.deallocate`,
  `hooks.reallocate`), but they take and return `Vec<u8>`, not raw pointers:
  use `hooks.alloc(size)`, `hooks.dealloc(buf)` and `hooks.realloc(buf, size, keep)`.
- The C code's `input_pointer`, `output_pointer`, `input_end` are NOT Rust
  pointers. Use a `usize` index into the buffer, or push into a `Vec<u8>`:
  ```c
  while (input_pointer < input_end) { c = *input_pointer; input_pointer++; }
  *output_pointer = c; output_pointer++;
  ```
  ```rust
  let mut i = start;
  while i < end {
      let c = input_buffer.content[i];
      i += 1;
  }
  output.push(c);
  ```
  A `usize` index into a slice is the Rust way to walk a C pointer.

Reply with exactly one ```rust code block containing the Rust implementations of the functions above."""


FUNC_PROMPT = """Translate ONE C function to safe, idiomatic Rust.

## 1. Rust contract - ALREADY WRITTEN, do not redefine these types
```rust
{contract}
```
The contract is EXHAUSTIVE: these are the only types, fields, constants and
helper functions that exist.

## 2. The function to implement (exact signature required)
```rust
{signature};
```

## 3. Rust code of the functions it calls (already translated - call them as they are)
```rust
{callee_code}
```

## 4. The C source of that one function (the behaviour you must preserve)
```c
{csource}
```

## 5. How to convert the C idioms
The Rust types keep the C field names, so most code translates directly:
- `item->type` -> `item.type_` ; `item->valuestring` -> `item.valuestring` ;
  `item->valueint` -> `item.valueint` ; `item->valuedouble` -> `item.valuedouble` ;
  `item->string` -> `item.string`.
- `item->child` -> `item.children.first()` (or `item.child()`).
- `item->next` -> the NEXT element of the `children` Vec. There is no `next` field.
- `item->prev` -> the PREVIOUS element of the `children` Vec.
- `array->child = item` (append) -> `array.children.push(item)`.
- `strcpy(dst, "lit")` -> `dst.copy_from_slice(b"lit")`.
- `strlen(s)` -> `s.len()`.
- `memcpy(d, s, n)` -> `d[..n].copy_from_slice(&s[..n])`.
- `'0'` (a C char literal) -> `b'0'` (a Rust u8). Same for every other literal.
- `INT_MAX` -> `i32::MAX as usize`.
- `NULL` -> `None`; `x == NULL` -> `x.is_none()` ONLY for an `Option`. A `&[u8]`
  parameter is never `None`, so never test it for null.
- A `char *` that the function allocates and returns -> `Option<Vec<u8>>`.
- A `char *` that the function writes into -> `&mut Vec<u8>`.
- `cJSON_Delete(item)` frees memory: in Rust nothing has to be done, the `Node`
  is dropped automatically, so those calls simply disappear.
- The C `hooks` struct fields exist in Rust (`hooks.allocate`, `hooks.deallocate`,
  `hooks.reallocate`), but they take and return `Vec<u8>`, not raw pointers:
  use `hooks.alloc(size)`, `hooks.dealloc(buf)` and `hooks.realloc(buf, size, keep)`.
- A C bitmask test `x & FLAG` -> `x & FLAG != 0`. There is no `.contains()` on an integer.
- The contract's methods need `()`: `item.child()` not `item.child`.
- The C code's `input_pointer`, `output_pointer`, `input_end` are NOT Rust
  pointers. Use a `usize` index into the buffer, or push into a `Vec<u8>`:
  ```c
  while (input_pointer < input_end) { c = *input_pointer; input_pointer++; }
  *output_pointer = c; output_pointer++;
  ```
  ```rust
  let mut i = start;
  while i < end {
      let c = input_buffer.content[i];
      i += 1;
  }
  output.push(c);
  ```
  A `usize` index into a slice is the Rust way to walk a C pointer.

Reply with exactly ONE ```rust code block containing ONLY this function."""

FUNC_REPAIR = """Your Rust function does not compile. Fix it.

## Compiler errors
```
{errors}
```

## Your current code
```rust
{code}
```

## The exact signature you must keep
```rust
{signature};
```

Rules for the fix:
- Fix every error. Do not leave any error in place.
- Keep the behaviour of the C original unchanged.
- Keep the same name and signature.
- If the error says a field or method does not exist, the contract does not
  have it: use the Rust equivalent instead (`Vec` methods, `Option`, slices).
- Reply with the COMPLETE corrected function in exactly one ```rust code block."""


def read_json(p):
    with open(p) as fh:
        return json.load(fh)


_analyzer = None


def extract_function_text(name: str) -> str:
    """The C source of one function (from the analyzer's brace-matched extractor)."""
    global _analyzer
    if _analyzer is None:
        import importlib.util
        spec = importlib.util.spec_from_file_location(
            "an", os.path.join(ROOT, "scripts", "20_analyze.py"))
        _analyzer = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(_analyzer)
    return _analyzer.extract_function_text(name)


def unit_c_source(fn_names: list[str]) -> str:
    import importlib.util
    spec = importlib.util.spec_from_file_location(
        "an", os.path.join(ROOT, "scripts", "20_analyze.py"))
    an = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(an)
    return "\n\n".join(an.extract_function_text(f) for f in fn_names)


def extract_rust(text: str) -> str | None:
    blocks = re.findall(r"```(?:rust|rs)?\s*\n(.*?)```", text, re.S)
    if not blocks:
        return None
    code = blocks[0]
    # strip a leading "mod x { ... }" wrapper if the model added one
    m = re.match(r"\s*(?:pub\s+)?mod\s+\w+\s*\{(.*)\}\s*$", code, re.S)
    if m:
        code = m.group(1)
    return code.strip()


BASELINE_RS = os.path.join(ROOT, "artifacts", "baseline", "c2rust", "src", "cJSON.rs")
_baseline_cache: list[str] | None = None


def baseline_code(fn_names: list[str]) -> str:
    """The c2rust (faithful, unsafe) translation of the unit's functions."""
    global _baseline_cache
    if _baseline_cache is None:
        if not os.path.isfile(BASELINE_RS):
            return ""
        _baseline_cache = open(BASELINE_RS, encoding="utf-8", errors="replace").read()
    src = _baseline_cache
    out = []
    for name in fn_names:
        m = re.search(r"(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+\"C\"\s+)?fn\s+" +
                      re.escape(name) + r"\s*\(", src)
        if not m:
            continue
        # include the preceding attribute lines (#[no_mangle] etc.)
        start = src.rfind("\n", 0, m.start())
        start = 0 if start == -1 else start + 1
        attrs = src[start:m.start()]
        if "#[no_mangle]" not in attrs:
            attrs = ""
        i = src.index("{", m.end())
        depth = 0
        for j in range(i, len(src)):
            if src[j] == "{":
                depth += 1
            elif src[j] == "}":
                depth -= 1
                if depth == 0:
                    out.append(attrs + src[m.start():j + 1])
                    break
    return "\n\n".join(out)


def normalise_pub(code: str) -> tuple[str, bool]:
    """Make every top-level `fn` public. Returns (code, changed).

    Visibility is a purely mechanical property, so fixing it here is safe;
    it is recorded per unit so the effect on the compile rate is visible.
    """
    changed = False
    out_lines = []
    for line in code.splitlines():
        m = re.match(r"^(\s*)fn\s+([A-Za-z_][A-Za-z0-9_]*)", line)
        if m and not line.lstrip().startswith("pub"):
            indent = m.group(1)
            line = f"{indent}pub fn {m.group(2)}" + line[m.end():]
            changed = True
        out_lines.append(line)
    return "\n".join(out_lines), changed


def write_scratch(crate: str, unit_code: str, sigs: dict, callees: list[str]) -> str:
    """A crate with the contract, stubs for the external callees, and the unit.

    Compiling the unit *in isolation* stops a failure in an earlier unit from
    poisoning every later unit's compile rate. The stubs have exactly the
    target signatures, so the unit is compiled against the same interface it
    will meet in the final crate.
    """
    src = os.path.join(crate, "src")
    os.makedirs(src, exist_ok=True)
    with open(os.path.join(src, "contract.rs"), "w") as fh:
        fh.write(contract_mod.CONTRACT_RS)
    with open(os.path.join(src, "stubs.rs"), "w") as fh:
        fh.write("// generated stubs for callees that live in other units\n")
        fh.write("use crate::contract::*;\n\n")
        for c in callees:
            s = sigs[c]
            fh.write(f"pub {s['rust_signature']} {{ unimplemented!(\"stub for {c}\") }}\n")
    mod = "unit_under_test"
    with open(os.path.join(src, mod + ".rs"), "w") as fh:
        fh.write("use crate::contract::*;\nuse crate::stubs::*;\n\n")
        fh.write(unit_code + "\n")
    with open(os.path.join(src, "lib.rs"), "w") as fh:
        fh.write("pub mod contract;\npub mod stubs;\n")
        fh.write(f"pub mod {mod};\n")
    with open(os.path.join(crate, "Cargo.toml"), "w") as fh:
        fh.write("[package]\nname = \"scratch\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n"
                 "[lib]\nname = \"scratch\"\npath = \"src/lib.rs\"\n")
    return crate


def write_crate(out_dir: str, units: list[str], code_by_unit: dict[str, str]) -> str:
    """(Re)write the crate from the contract + the given units."""
    src = os.path.join(out_dir, "src")
    os.makedirs(src, exist_ok=True)
    with open(os.path.join(src, "contract.rs"), "w") as fh:
        fh.write(contract_mod.CONTRACT_RS)
    mods = []
    for u in units:
        code = code_by_unit.get(u)
        if code is None:
            continue
        mod_name = re.sub(r"[^0-9a-zA-Z]+", "_", u).strip("_").lower()
        fn = os.path.join(src, f"{mod_name}.rs")
        # every unit module gets the contract's types in scope; the model is
        # told not to write `use` statements, so the assembler adds it.
        alias_lines = []
        sched_p = os.path.join(ROOT, "artifacts", "analysis", "schedule.json")
        sigs_p = os.path.join(ROOT, "artifacts", "analysis", "target_signatures.json")
        if os.path.exists(sched_p) and os.path.exists(sigs_p):
            sched_data = json.load(open(sched_p))
            sigs_data = json.load(open(sigs_p))
            for entry in sched_data.get("units", []):
                if entry.get("unit") == u:
                    for fname in entry.get("functions", []):
                        if fname.startswith("cJSON_") and fname in sigs_data:
                            rn = sigs_data[fname]["rust_name"]
                            cn = sigs_data[fname]["c_name"]
                            if rn != cn:
                                alias_lines.append(f"pub use {rn} as {cn};")
        alias_block = "\n" + "\n".join(alias_lines) if alias_lines else ""
        with open(fn, "w") as fh:
            fh.write("//@unit: " + u + "\n")
            fh.write("#![allow(unused_imports, unused_variables, dead_code)]\n")
            fh.write("use crate::contract::*;\n")
            fh.write("use crate::*;\n\n")
            fh.write(code + "\n" + alias_block + "\n")
        mods.append(f"pub mod {mod_name};\npub use {mod_name}::*;")
    with open(os.path.join(src, "lib.rs"), "w") as fh:
        fh.write("// Generated by 30_translate.py - do not edit by hand.\n")
        fh.write("#![allow(unused_imports, unused_variables, dead_code, ambiguous_glob_reexports)]\n")
        fh.write("pub mod contract;\n")
        fh.write("pub use contract::*;\n")
        for m in mods:
            fh.write(m + "\n")
    with open(os.path.join(out_dir, "Cargo.toml"), "w") as fh:
        fh.write(
            "[package]\nname = \"cjson_rs\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n"
            "[lib]\nname = \"cjson_rs\"\npath = \"src/lib.rs\"\n\n"
            "[profile.release]\ndebug = true\n")
    return out_dir


def cargo_check(crate: str) -> tuple[bool, str]:
    """Run cargo check and return (ok, human-readable log).

    The *human* format is used on purpose: it contains the compiler's
    "help: consider ..." suggestions, which are the most useful signal for a
    small model when it is asked to repair a compile error.
    """
    env = dict(os.environ)
    env["PATH"] = os.path.expanduser("~/.cargo/bin") + ":" + env.get("PATH", "")
    p = subprocess.run(["cargo", "check", "--color=never"],
                       cwd=crate, capture_output=True, text=True, env=env, timeout=900)
    return p.returncode == 0, (p.stdout + p.stderr)


def error_lines(log: str) -> list[str]:
    return [ln for ln in log.splitlines()
            if ": error" in ln or ln.startswith("error")]


REPAIR = """Your Rust code does not compile. Fix it.

## Compiler errors
```
{errors}
```

## Your current code
```rust
{code}
```

## The functions you must implement (exact signatures, do not change them)
```rust
{signatures}
```

Rules for the fix:
- Fix every error. Do not leave any error in place.
- Keep the behaviour of the C original unchanged.
- Keep the same function names and signatures.
- If the error says a field or method does not exist, the contract does not have it:
  use the Rust equivalent instead (`Vec` methods, `Option`, slices).
- Reply with the COMPLETE corrected code (all the functions listed above) in
  exactly one ```rust code block. No prose."""


def build_func_repair_prompt(code: str, log: str, signature: str, mod_file: str) -> str:
    blocks, cur = [], []
    for ln in log.splitlines():
        if ": error" in ln or ln.startswith("error"):
            if cur:
                blocks.append("\n".join(cur))
            cur = [ln]
        elif cur:
            cur.append(ln)
    if cur:
        blocks.append("\n".join(cur))
    own = [b for b in blocks if mod_file in b]
    other = [b for b in blocks if mod_file not in b]
    errs = "\n\n".join((own + other[:2])[:20])
    if other:
        errs += "\n\n// (other modules also have errors; they are NOT yours to fix)"
    return FUNC_REPAIR.format(errors=errs or "(no error text captured)", code=code,
                              signature=signature)


def build_function_prompt(signature: str, callee_code: str, csrc: str,
                          contract: str) -> str:
    return FUNC_PROMPT.format(contract=contract, signature=signature,
                               callee_code=callee_code or "(none)",
                               csource=csrc)


def build_repair_prompt(code: str, log: str, sigs_txt: str, mod_file: str) -> str:
    """Only show the errors that belong to THIS unit.

    Showing errors from other modules would make the model try to fix code it
    does not own.
    """
    # keep whole error *blocks* (the error plus its "help:" notes) that
    # belong to this unit; those notes are the actionable part.
    blocks, cur = [], []
    for ln in log.splitlines():
        if ": error" in ln or ln.startswith("error"):
            if cur:
                blocks.append("\n".join(cur))
            cur = [ln]
        elif cur:
            cur.append(ln)
    if cur:
        blocks.append("\n".join(cur))
    own = [b for b in blocks if mod_file in b]
    other = [b for b in blocks if mod_file not in b]
    errs = "\n\n".join((own + other[:2])[:20])
    if other:
        errs += "\n\n// (other modules also have errors; they are NOT yours to fix)"
    return REPAIR.format(errors=errs or "(no error text captured)", code=code,
                          signatures=sigs_txt)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", required=True, help="model id passed to the local server")
    ap.add_argument("--out", required=True, help="output crate directory")
    ap.add_argument("--port", default="8080")
    ap.add_argument("--limit", type=int, default=0, help="only translate the first N units")
    ap.add_argument("--only", default="", help="comma separated unit names to translate")
    ap.add_argument("--resume", action="store_true",
                    help="reuse an existing candidate for a unit when it already compiles")
    ap.add_argument("--max-attempts", type=int, default=5,
                    help="max model calls per unit (1 initial + N-1 repairs)")
    ap.add_argument("--baseline", choices=["prompt", "none"], default="prompt",
                    help="whether the c2rust scaffold is shown to the model")
    args = ap.parse_args()

    sched = read_json(os.path.join(A, "schedule.json"))
    sigs = read_json(os.path.join(A, "target_signatures.json"))
    units = sched["units"]
    if args.only:
        want = set(args.only.split(","))
        units = [u for u in units if u["unit"] in want]
    elif args.limit:
        units = units[: args.limit]

    out_dir = os.path.join(ROOT, args.out)
    cand_dir = os.path.join(out_dir, "candidates")
    os.makedirs(cand_dir, exist_ok=True)

    cfg = LLMConfig(backend="local", model=args.model,
                    url=f"http://127.0.0.1:{args.port}/v1/chat/completions",
                    temperature=0.0, max_tokens=4096)

    # the Rust signatures of every function, used as "already translated callees"
    all_rust_sigs = {c: s["rust_signature"] for c, s in sigs.items()}

    code_by_unit: dict[str, str] = {}
    results = []
    translated_so_far: list[str] = []

    for u in units:
        name = u["unit"]
        fns = u["functions"]
        print(f"\n=== {name} ({len(fns)} fns) ===", flush=True)

        # ---- resume: reuse an existing candidate that already compiles ----
        cand_path = os.path.join(cand_dir, f"{name}.rs")
        if args.resume and os.path.isfile(cand_path):
            prior = open(cand_path, encoding="utf-8", errors="replace").read()
            prior_code = "\n".join(ln for ln in prior.splitlines()
                                    if not ln.startswith("//"))
            if prior_code.strip():
                ext = sorted({c for c in u["calls_earlier_units"] + u["calls_later_units"]
                              if c in sigs and c not in fns})
                write_scratch(os.path.join(out_dir, "scratch"), prior_code, sigs, ext)
                pok, _ = cargo_check(os.path.join(out_dir, "scratch"))
                if pok:
                    code_by_unit[name] = prior_code
                    translated_so_far.append(name)
                    results.append({
                        "unit": name, "model": args.model,
                        "prompt_sha256": "resumed", "prompt_tokens": 0,
                        "completion_tokens": 0, "wall_seconds": 0,
                        "has_code": True, "pub_normalised": None,
                        "compiles_first_try": None, "compiles_after_repair": True,
                        "attempts": 0, "first_try_errors": [], "history": [],
                        "resumed": True,
                    })
                    print(f"  resumed from a previous candidate that compiles")
                    continue

        # signatures of the functions this unit must implement
        sig_lines = []
        for f in fns:
            sig_lines.append(sigs[f]["rust_signature"] + ";")
        signatures = "\n".join(sig_lines)

        # callees that live in other units
        callee_names = sorted({c for c in u["calls_earlier_units"] + u["calls_later_units"]
                               if c in sigs and c not in fns})
        callees = "\n".join(all_rust_sigs[c] + ";" for c in callee_names) or "(none)"

        macro_notes = "\n".join(
            f"- `{m['name']}`: {m['rust_note']}"
            + (f"  [C expansion: {m['expansion']}]" if m.get("expansion") else "")
            for m in u.get("macro_guidance", [])) or "(none)"

        csrc = unit_c_source(fns)
        base = baseline_code(fns)
        ctext = read_json(os.path.join(A, "callgraph.json"))["functions"]
        # pick the contract blocks this unit actually needs
        need = ["consts", "libc_helpers"]
        if any("parse_buffer" in ctext[f]["params"][i]["type"] for f in fns
               for i in range(len(ctext[f]["params"]))) or "parse_buffer" in csrc:
            need.append("parsebuf")
        if "printbuffer" in csrc or "PrintBuffer" in csrc:
            need.append("printbuf")
        if "internal_hooks" in csrc or "Hooks" in csrc:
            need += ["hooks", "init_hooks", "try_alloc"]
        if "global_error" in csrc or "GetErrorPtr" in fns or "set_error" in csrc:
            need.append("error_api")
        ctext_contract = contract_mod.contract_for(need)

        prompt = PROMPT.format(contract=ctext_contract, examples=WORKED_EXAMPLES,
                                signatures=signatures,
                                callees=callees, macro_notes=macro_notes,
                                baseline=(base if args.baseline == "prompt" else "(not shown)"),
                                csource=csrc)
        t0 = time.time()
        r = ask(prompt, system=SYSTEM, cfg=cfg, tag=f"translate:{name}")
        dt = time.time() - t0

        code = extract_rust(r.text)
        pub_fixed = False
        if code:
            code, pub_fixed = normalise_pub(code)

        mod_file = "unit_under_test.rs"

        # ---- compile-feedback repair loop ----
        # The candidate is kept at every attempt; the best (fewest errors) wins.
        attempts = 1
        first_try_ok = False
        first_try_errors = None
        best_code = code
        best_err = 10 ** 9
        history = [{"attempt": 1, "prompt_sha256": r.prompt_sha256,
                    "prompt_tokens": r.usage.get("prompt_tokens"),
                    "completion_tokens": r.usage.get("completion_tokens"),
                    "wall_seconds": round(dt, 1)}]

        # The unit is checked in ISOLATION (contract + stubs for its external
        # callees). Otherwise a failure in an earlier unit would make every
        # later unit look broken.
        scratch = os.path.join(out_dir, "scratch")
        ext_callees = sorted({c for c in callee_names if c not in fns})

        def check_unit(candidate: str) -> tuple[bool, str]:
            write_scratch(scratch, candidate, sigs, ext_callees)
            return cargo_check(scratch)

        if code:
            ok, log = check_unit(code)
            errs = error_lines(log)
            first_try_ok = ok
            first_try_errors = errs[:20]
            best_err = len(errs)

            while not ok and attempts < args.max_attempts:
                rp = build_repair_prompt(code, log, signatures, mod_file)
                t1 = time.time()
                r2 = ask(rp, system=SYSTEM, cfg=cfg, tag=f"repair:{name}")
                d1 = time.time() - t1
                new_code = extract_rust(r2.text)
                if not new_code:
                    history.append({"attempt": attempts + 1, "error": "no code block in reply",
                                    "wall_seconds": round(d1, 1)})
                    break
                new_code, _ = normalise_pub(new_code)
                attempts += 1
                history.append({"attempt": attempts, "prompt_sha256": r2.prompt_sha256,
                                "prompt_tokens": r2.usage.get("prompt_tokens"),
                                "completion_tokens": r2.usage.get("completion_tokens"),
                                "wall_seconds": round(d1, 1)})
                ok2, log2 = check_unit(new_code)
                errs2 = error_lines(log2)
                if len(errs2) < best_err:
                    best_err = len(errs2)
                    best_code = new_code
                code = new_code
                log = log2
                ok = ok2
                if ok:
                    break

            # keep the best candidate we saw
            if best_code and best_err < len(error_lines(log)):
                code = best_code
                ok, log = check_unit(code)
        cand_path = os.path.join(cand_dir, f"{name}.rs")
        with open(cand_path, "w") as fh:
            fh.write(f"// model: {args.model}\n// prompt_sha256: {r.prompt_sha256}\n"
                     f"// tokens in/out: {r.usage.get('prompt_tokens')}/"
                     f"{r.usage.get('completion_tokens')}\n"
                     f"// wall: {dt:.1f}s\n"
                     f"// pub-normalised: {pub_fixed}\n")
            fh.write((code or "") + "\n")
        if code is None:
            with open(cand_path + ".raw.txt", "w") as fh:
                fh.write(r.text)

        code_by_unit[name] = code or ""
        errs = error_lines(log)
        print(f"  tokens in/out: {r.usage.get('prompt_tokens')}/{r.usage.get('completion_tokens')}"
              f"  wall: {dt:.1f}s")
        print(f"  compile-first-try: {'PASS' if first_try_ok else 'FAIL'}  "
              f"| after {attempts} attempt(s): {'PASS' if ok else 'FAIL'}", flush=True)
        if not ok:
            for e in errs[:6]:
                print(f"      {e[:180]}")
            with open(os.path.join(cand_dir, f"{name}.errors.txt"), "w") as fh:
                fh.write(log)

        results.append({
            "unit": name,
            "model": args.model,
            "prompt_sha256": r.prompt_sha256,
            "prompt_tokens": r.usage.get("prompt_tokens"),
            "completion_tokens": r.usage.get("completion_tokens"),
            "wall_seconds": round(dt, 1),
            "has_code": code is not None,
            "pub_normalised": pub_fixed,
            "compiles_first_try": first_try_ok,
            "compiles_after_repair": ok,
            "attempts": attempts,
            "first_try_errors": first_try_errors,
            "history": history,
            "error_count": len(errs),
            "errors": errs[:20],
            "candidate": os.path.relpath(cand_path, ROOT),
        })
        translated_so_far.append(name)

    # final assembly
    write_crate(out_dir, [u["unit"] for u in sched["units"]], code_by_unit)
    ok, log = cargo_check(out_dir)
    fresh = [r for r in results if not r.get("resumed")]
    n_pass = sum(1 for r in fresh if r["compiles_first_try"])
    n_final = sum(1 for r in fresh if r["compiles_after_repair"])
    summary = {
        "model": args.model,
        "out": args.out,
        "units": len(results),
        "compile_first_try_pass": n_pass,
        "compile_first_try_rate": f"{n_pass}/{len(fresh)}",
        "units_attempted": len(fresh),
        "units_resumed": len(results) - len(fresh),
        "compiles_after_repair_pass": n_final,
        "compiles_after_repair_rate": f"{n_final}/{len(fresh)}",
        "total_model_calls": sum(r["attempts"] for r in results),
        "final_crate_compiles": ok,
        "results": results,
    }
    with open(os.path.join(out_dir, "translation_results.json"), "w") as fh:
        json.dump(summary, fh, indent=2)
    print(f"\n=== {args.model}: compile-first-try {n_pass}/{len(fresh)}; "
          f"after repair {n_final}/{len(fresh)}; "
          f"model calls {sum(r['attempts'] for r in results)}; "
          f"final crate compiles: {ok} ===")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
