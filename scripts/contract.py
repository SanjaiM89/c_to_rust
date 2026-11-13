#!/usr/bin/env python3
"""
contract.py - the symbolic skeleton of the Rust port.

The pipeline is neurosymbolic: the *symbol* is this contract (shared types,
constants and the target signature of every function), the *neural* part is the
local model filling in the function bodies.

Why a contract is necessary
---------------------------
Each unit is translated independently by a 4B model. Without a fixed skeleton
the units would not compose: two units would invent two different `Node`
types. The contract is therefore written once, by hand, and handed to every
prompt.

Design decisions that encode cJSON's real behaviour
---------------------------------------------------
* `Node.children: Vec<Node>` replaces the C `child`/`next`/`prev` chain.
* Strings are `Vec<u8>` / `&[u8]`, NOT `String`/`&str`: cJSON accepts and
  preserves arbitrary bytes, and `String` would be a UTF-8 invariant
  violation (the corpus contains invalid UTF-8 on purpose).
* The `type` field stays a bitmask (`u32`) so the exact C flag semantics
  (`cJSON_IsTrue` tests `type == 1<<1`, `cJSON_SetBoolValue` does bit surgery)
  are preserved.
* `NULL` returns become `Option<T>`; `cJSON_bool` returns become `bool`.
* The custom-allocator hook is kept, because the upstream test suite exercises
  allocation failure (`cjson_add.c` installs a failing allocator and expects
  NULL results). It is modelled with a Rust-native hook, no raw pointers.
* References (`cJSON_CreateObjectReference` etc.) own a *clone* of the wrapped
  node. C shares the pointer; the divergence is measured in Phase 4.

Output:
  artifacts/analysis/rust_contract.rs       the shared substrate (types, consts)
  artifacts/analysis/target_signatures.json  C name -> Rust name + target signature
"""
from __future__ import annotations

import json
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
A = os.path.join(ROOT, "artifacts", "analysis")

# --------------------------------------------------------------------------- #
# the shared substrate, written by hand
# --------------------------------------------------------------------------- #
CONTRACT_RS = r'''//! Shared substrate for the cJSON -> Rust port.
//!
//! This file is the *symbolic* part of the pipeline: hand-written, and handed
//! unchanged to every translation unit so that independently generated units
//! compose. The model only ever writes function bodies.
//!
//! DESIGN RULE: every Rust name below is deliberately as close as possible to
//! the C name it replaces (`type` -> `type_` only because `type` is a Rust
//! keyword). That way the translation is a *type* change, not a *rename*, and
//! a small model cannot drift from the contract by inventing names.

// ---------------------------------------------------------------------------
// type flags (identical bit values to the C header)
// ---------------------------------------------------------------------------
//@BLOCK:consts
pub const CJSON_INVALID: u32 = 0;
pub const CJSON_FALSE: u32 = 1 << 0;
pub const CJSON_TRUE: u32 = 1 << 1;
pub const CJSON_NULL: u32 = 1 << 2;
pub const CJSON_NUMBER: u32 = 1 << 3;
pub const CJSON_STRING: u32 = 1 << 4;
pub const CJSON_ARRAY: u32 = 1 << 5;
pub const CJSON_OBJECT: u32 = 1 << 6;
pub const CJSON_RAW: u32 = 1 << 7;
pub const CJSON_IS_REFERENCE: u32 = 256;
pub const CJSON_STRING_IS_CONST: u32 = 512;

/// Maximum nesting depth accepted while parsing.
pub const CJSON_NESTING_LIMIT: usize = 1000;
/// Maximum number of items visited while duplicating / comparing.
pub const CJSON_CIRCULAR_LIMIT: usize = 10000;

// ---------------------------------------------------------------------------
// allocator hooks
// ---------------------------------------------------------------------------
//@BLOCK:hooks
/// Allocate `size` bytes. Returning `None` means the allocation failed.
pub type AllocFn = fn(size: usize) -> Option<Vec<u8>>;
/// Release a buffer previously returned by an `AllocFn`.
pub type DeallocFn = fn(buffer: Vec<u8>);
/// Resize a buffer. Returning `None` means the reallocation failed.
pub type ReallocFn = fn(buffer: Vec<u8>, size: usize) -> Option<Vec<u8>>;

/// The allocator hooks, mirroring the C `internal_hooks` struct.
///
/// `None` means "use the default Rust allocator" (`Vec`).
#[derive(Clone, Copy, Debug)]
pub struct Hooks {
    pub allocate: Option<AllocFn>,
    pub deallocate: Option<DeallocFn>,
    pub reallocate: Option<ReallocFn>,
}

impl Default for Hooks {
    fn default() -> Self {
        Hooks { allocate: None, deallocate: None, reallocate: None }
    }
}

impl Hooks {
    /// Allocate `size` bytes through these hooks.
    pub fn alloc(&self, size: usize) -> Option<Vec<u8>> {
        match self.allocate {
            Some(f) => f(size),
            None => Some(vec![0u8; size]),
        }
    }

    /// Release a buffer through these hooks.
    pub fn dealloc(&self, buffer: Vec<u8>) {
        if let Some(f) = self.deallocate {
            f(buffer);
        }
    }

    /// Resize a buffer through these hooks, preserving its first `keep` bytes.
    pub fn realloc(&self, buffer: Vec<u8>, size: usize, keep: usize) -> Option<Vec<u8>> {
        match self.reallocate {
            Some(f) => f(buffer, size),
            None => {
                let mut new_buffer = vec![0u8; size];
                let n = keep.min(buffer.len()).min(size);
                new_buffer[..n].copy_from_slice(&buffer[..n]);
                Some(new_buffer)
            }
        }
    }
}

thread_local! {
    /// The currently installed hooks, mirroring the C `global_hooks` global.
    pub static GLOBAL_HOOKS: std::cell::RefCell<Hooks> =
        const { std::cell::RefCell::new(Hooks { allocate: None, deallocate: None, reallocate: None }) };
}

/// Install custom allocator hooks, or restore the defaults with `None`.
pub fn contract_init_hooks(hooks: Option<&Hooks>) {
    GLOBAL_HOOKS.with(|h| {
        *h.borrow_mut() = match hooks {
            Some(x) => *x,
            None => Hooks::default(),
        };
    });
}

/// The currently installed hooks, mirroring the C `global_hooks` global.
pub fn global_hooks() -> Hooks {
    GLOBAL_HOOKS.with(|h| *h.borrow())
}

/// Ask the active hooks whether an allocation of `size` bytes may succeed.
///
/// Returns `true` when the allocation may proceed. Mirrors the C behaviour
/// where a failing allocator makes the whole operation fail.
pub fn try_alloc(size: usize) -> bool {
    global_hooks().alloc(size).is_some()
}

// ---------------------------------------------------------------------------
// nodes
// ---------------------------------------------------------------------------
//@BLOCK:node
/// A JSON node. Mirrors the C `cJSON` struct.
///
/// `valuestring` and `string` are RAW BYTES (`Vec<u8>`), exactly like the C
/// `char *` they replace: cJSON accepts and preserves arbitrary bytes, so
/// `String`/`&str` would be a UTF-8 invariant violation.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Node {
    /// The C `type` field: a bitmask of the `CJSON_*` flags.
    pub type_: u32,
    /// The C `valuestring` field: payload of a string or raw node.
    pub valuestring: Option<Vec<u8>>,
    /// The C `valueint` field.
    pub valueint: i32,
    /// The C `valuedouble` field.
    pub valuedouble: f64,
    /// The C `string` field: the object key.
    pub string: Option<Vec<u8>>,
    /// The C `child`/`next`/`prev` chain, as a plain list of children.
    pub children: Vec<Node>,
}

impl Node {
    /// A new node of the given type, with no payload.
    pub fn new(type_: u32) -> Node {
        Node { type_, ..Default::default() }
    }

    /// The first child, mirroring the C `item->child` pointer.
    pub fn child(&self) -> Option<&Node> {
        self.children.first()
    }

    /// The first child, mutably, mirroring the C `item->child` pointer.
    pub fn child_mut(&mut self) -> Option<&mut Node> {
        self.children.first_mut()
    }

    /// Append a child, mirroring the C `suffix_object` / append behaviour.
    pub fn push_child(&mut self, child: Node) {
        self.children.push(child);
    }

    /// Iterate over the children, mirroring the C sibling loop
    /// `for (item = item->child; item != NULL; item = item->next)`.
    pub fn iter_children(&self) -> std::slice::Iter<'_, Node> {
        self.children.iter()
    }

    /// Iterate over the children mutably.
    pub fn iter_children_mut(&mut self) -> std::slice::IterMut<'_, Node> {
        self.children.iter_mut()
    }

    /// The child at `index`, mirroring the C `get_array_item`.
    pub fn child_at(&self, index: usize) -> Option<&Node> {
        self.children.get(index)
    }

    /// The child at `index`, mutably.
    pub fn child_at_mut(&mut self, index: usize) -> Option<&mut Node> {
        self.children.get_mut(index)
    }

    /// The position of the child equal to `other`, mirroring the C
    /// `cJSON_DetachItemViaPointer` search.
    pub fn position_of(&self, other: &Node) -> Option<usize> {
        self.children.iter().position(|c| c == other)
    }
}

// ---------------------------------------------------------------------------
// parse / print buffers
// ---------------------------------------------------------------------------
//@BLOCK:parsebuf
/// Parsing cursor over the input bytes. Mirrors the C `parse_buffer`.
#[derive(Debug, Clone)]
pub struct ParseBuffer<'a> {
    /// The whole input.
    pub content: &'a [u8],
    /// Length of the input.
    pub length: usize,
    /// Current read position.
    pub offset: usize,
    /// Current recursion depth.
    pub depth: usize,
}

impl<'a> ParseBuffer<'a> {
    /// A buffer over `content`.
    pub fn new(content: &'a [u8]) -> ParseBuffer<'a> {
        ParseBuffer { content, length: content.len(), offset: 0, depth: 0 }
    }
}

//@BLOCK:printbuf
/// Output buffer used by the printer. Mirrors the C `printbuffer`.
///
/// The content lives in `buffer[0..offset]` and is NUL terminated at
/// `buffer[offset]` (the C code relies on that in `update_offset`). `length`
/// is the allocated capacity. `noalloc` forbids growing, which is the
/// `cJSON_PrintPreallocated` mode.
#[derive(Debug, Clone, Default)]
pub struct PrintBuffer {
    /// The output bytes.
    pub buffer: Vec<u8>,
    /// Allocated capacity.
    pub length: usize,
    /// Write cursor.
    pub offset: usize,
    /// Current nesting depth.
    pub depth: usize,
    /// When true the buffer may not grow.
    pub noalloc: bool,
    /// True for a formatted (pretty) print.
    pub format: bool,
    /// The allocator hooks used when growing.
    pub hooks: Hooks,
}

// ---------------------------------------------------------------------------
// small helpers the C code used from libc
// ---------------------------------------------------------------------------
//@BLOCK:libc_helpers
/// Lowercase an ASCII byte. The C code used `tolower` from libc.
pub fn to_lower(b: u8) -> u8 {
    b.to_ascii_lowercase()
}

/// Uppercase an ASCII byte. The C code used `toupper` from libc.
pub fn to_upper(b: u8) -> u8 {
    b.to_ascii_uppercase()
}

// ---------------------------------------------------------------------------
// error reporting
// ---------------------------------------------------------------------------
//@BLOCK:error_api
thread_local! {
    /// The most recent parse error, mirroring the C `global_error` struct.
    static GLOBAL_ERROR: std::cell::RefCell<Option<(Vec<u8>, usize)>> =
        const { std::cell::RefCell::new(None) };
}

/// Record a parse error at `offset` with the given message.
pub fn set_error(message: &[u8], offset: usize) {
    GLOBAL_ERROR.with(|e| {
        *e.borrow_mut() = Some((message.to_vec(), offset));
    });
}

/// Clear the recorded parse error.
pub fn clear_error() {
    GLOBAL_ERROR.with(|e| {
        *e.borrow_mut() = None;
    });
}

/// The most recent parse error message, if any.
pub fn contract_get_error_ptr() -> Option<&'static [u8]> {
    GLOBAL_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map(|(msg, _)| unsafe {
                // SAFETY: the message lives in a thread_local that is never
                // freed while the thread is alive; only immutable bytes are read.
                std::slice::from_raw_parts(msg.as_ptr(), msg.len())
            })
    })
}

/// The offset at which the most recent parse error occurred.
pub fn get_error_position() -> Option<usize> {
    GLOBAL_ERROR.with(|e| e.borrow().as_ref().map(|(_, pos)| *pos))
}
'''



# --------------------------------------------------------------------------- #
# C name -> Rust name
# --------------------------------------------------------------------------- #
SPECIAL_RUST_NAMES = {
    "cJSON_strdup": "cjson_strdup",
    "cJSON_Duplicate_rec": "cjson_duplicate_rec",
    "cJSON_malloc": "cjson_malloc",
    "cJSON_free": "cjson_free",
    "get_object_item": "get_object_item_internal",
    "get_array_item": "get_array_item_internal",
    "print": "print_internal",
    "add_item_to_array": "add_item_to_array_internal",
    "add_item_to_object": "add_item_to_object_internal",
    "replace_item_in_object": "replace_item_in_object_internal",
}


def contract_blocks() -> dict[str, str]:
    """Split CONTRACT_RS into selectable blocks."""
    blocks: dict[str, list[str]] = {}
    cur = "misc"
    for line in CONTRACT_RS.splitlines(keepends=True):
        if line.startswith("//@BLOCK:"):
            cur = line[len("//@BLOCK:"):].strip()
            blocks.setdefault(cur, [])
            continue
        blocks.setdefault(cur, []).append(line)
    return {k: "".join(v) for k, v in blocks.items()}


def contract_for(names: list[str]) -> str:
    """Concatenate the named contract blocks (plus the const block if asked)."""
    bl = contract_blocks()
    want = set(names)
    if "consts" not in want:
        # always include the type flags, they are cheap and always needed
        want.add("consts")
    out = []
    for name in ["consts", "node", "parsebuf", "printbuf", "hooks",
                 "init_hooks", "try_alloc", "libc_helpers", "error_api"]:
        if name in want and name in bl:
            out.append(bl[name].rstrip() + "\n")
    return "\n".join(out)


def rust_name(c_name: str) -> str:
    if c_name in SPECIAL_RUST_NAMES:
        return SPECIAL_RUST_NAMES[c_name]
    if c_name.startswith("cJSON_"):
        rest = c_name[len("cJSON_"):]
        if not rest:
            return "cjson"
        # camelCase / PascalCase -> snake_case
        s = re.sub(r"(.)([A-Z][a-z]+)", r"\1_\2", rest)
        s = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", s)
        return s.lower()
    return c_name


# --------------------------------------------------------------------------- #
# C type -> Rust type
# --------------------------------------------------------------------------- #
def rust_type(ct: str, *, ret: bool = False, fname: str = "") -> str:
    """Map a C type string onto the intended Rust type.

    `ret` selects the return-type mapping; for return types a pointer means
    "owned pointer" unless the function is known to return a borrowed one.
    """
    t = " ".join(ct.split())
    if t in ("void", ""):
        return "()" if ret else "()"
    if t == "cJSON_bool":
        return "bool"
    if t == "int":
        return "i32"
    if t == "unsigned int" or t == "unsigned":
        return "u32"
    if t == "double":
        return "f64"
    if t == "size_t":
        return "usize"
    if t == "unsigned char":
        return "u8"
    if t == "char":
        return "u8"

    # pointer types
    if t.endswith("**"):
        if "char" in t:
            return "Option<&mut usize>" if "const" in t else "Option<&mut Vec<u8>>"
        return "Option<&mut Vec<u8>>"

    if "*" in t:
        base = t.replace("*", "").replace("const", "").strip()
        is_const = "const" in t
        if base == "cJSON":
            if ret:
                return "Option<Node>"
            return "&Node" if is_const else "&mut Node"
        if base in ("char", "unsigned char"):
            if ret:
                return "Option<Vec<u8>>"
            return "&[u8]" if is_const else "&mut [u8]"
        if base == "void":
            return "&mut [u8]"
        if base == "parse_buffer":
            return "&mut ParseBuffer<'_>"
        if base == "printbuffer":
            return "&mut PrintBuffer"
        if base == "internal_hooks":
            return "&Hooks"
        if base == "cJSON_Hooks":
            return "&Hooks"
        if base == "void":
            return "&mut [u8]"
    return "()"


# --------------------------------------------------------------------------- #
# per-function overrides (where the generic mapping is wrong)
# --------------------------------------------------------------------------- #
# function -> (rust_return_type, [(c_param_name, rust_param_type), ...])
OVERRIDES: dict[str, tuple[str, list[tuple[str, str]]]] = {
    "cJSON_GetErrorPtr": ("Option<&'static [u8]>", []),
    "cJSON_Version": ("&'static str", []),
    "cJSON_InitHooks": ("()", [("hooks", "Option<&Hooks>")]),
    "cJSON_strdup": ("Option<Vec<u8>>", [
        ("string", "&[u8]"), ("hooks", "&Hooks")]),
    "cJSON_New_Item": ("Option<Node>", [("hooks", "&Hooks")]),
    "cJSON_Delete": ("()", [("item", "Node")]),
    "cJSON_Parse": ("Option<Node>", [("value", "&[u8]")]),
    "cJSON_ParseWithOpts": ("Option<Node>", [
        ("value", "&[u8]"), ("return_parse_end", "Option<&mut usize>"),
        ("require_null_terminated", "bool")]),
    "cJSON_ParseWithLengthOpts": ("Option<Node>", [
        ("value", "&[u8]"), ("buffer_length", "usize"),
        ("return_parse_end", "Option<&mut usize>"),
        ("require_null_terminated", "bool")]),
    "cJSON_ParseWithLength": ("Option<Node>", [
        ("value", "&[u8]"), ("buffer_length", "usize")]),
    "cJSON_Print": ("Option<Vec<u8>>", [("item", "&Node")]),
    "cJSON_PrintUnformatted": ("Option<Vec<u8>>", [("item", "&Node")]),
    "cJSON_PrintBuffered": ("Option<Vec<u8>>", [
        ("item", "&Node"), ("prebuffer", "usize"), ("fmt", "bool")]),
    "cJSON_PrintPreallocated": ("bool", [
        ("item", "&Node"), ("buffer", "&mut [u8]"), ("length", "i32"),
        ("format", "bool")]),
    "cJSON_Minify": ("()", [("json", "&mut Vec<u8>")]),
    "cJSON_GetStringValue": ("Option<&[u8]>", [("item", "Option<&Node>")]),
    "cJSON_GetNumberValue": ("f64", [("item", "Option<&Node>")]),
    "cJSON_GetArraySize": ("i32", [("array", "Option<&Node>")]),
    "cJSON_GetArrayItem": ("Option<&Node>", [("array", "Option<&Node>"), ("index", "i32")]),
    "cJSON_GetObjectItem": ("Option<&'a Node>", [
        ("object", "Option<&'a Node>"), ("string", "&[u8]")]),
    "cJSON_GetObjectItemCaseSensitive": ("Option<&'a Node>", [
        ("object", "Option<&'a Node>"), ("string", "&[u8]")]),
    "cJSON_HasObjectItem": ("bool", [("object", "Option<&Node>"), ("string", "&[u8]")]),
    "cJSON_IsInvalid": ("bool", [("item", "Option<&Node>")]),
    "cJSON_IsFalse": ("bool", [("item", "Option<&Node>")]),
    "cJSON_IsTrue": ("bool", [("item", "Option<&Node>")]),
    "cJSON_IsBool": ("bool", [("item", "Option<&Node>")]),
    "cJSON_IsNull": ("bool", [("item", "Option<&Node>")]),
    "cJSON_IsNumber": ("bool", [("item", "Option<&Node>")]),
    "cJSON_IsString": ("bool", [("item", "Option<&Node>")]),
    "cJSON_IsArray": ("bool", [("item", "Option<&Node>")]),
    "cJSON_IsObject": ("bool", [("item", "Option<&Node>")]),
    "cJSON_IsRaw": ("bool", [("item", "Option<&Node>")]),
    "cJSON_Compare": ("bool", [
        ("item1", "Option<&Node>"), ("item2", "Option<&Node>"), ("case_sensitive", "bool")]),
    "cJSON_Duplicate": ("Option<Node>", [("item", "&Node"), ("recursion", "bool")]),
    "cJSON_SetValuestring": ("Option<Vec<u8>>", [("object", "&mut Node"), ("valuestring", "&[u8]")]),
    "cJSON_SetNumberHelper": ("f64", [("object", "&mut Node"), ("number", "f64")]),
    "cJSON_AddItemToArray": ("bool", [("array", "&mut Node"), ("item", "Node")]),
    "cJSON_AddItemToObject": ("bool", [
        ("object", "&mut Node"), ("string", "&[u8]"), ("item", "Node")]),
    "cJSON_AddItemToObjectCS": ("bool", [
        ("object", "&mut Node"), ("string", "&[u8]"), ("item", "Node")]),
    "cJSON_AddItemReferenceToArray": ("bool", [("array", "&mut Node"), ("item", "Node")]),
    "cJSON_AddItemReferenceToObject": ("bool", [
        ("object", "&mut Node"), ("string", "&[u8]"), ("item", "Node")]),
    "cJSON_AddNullToObject": ("Option<Node>", [("object", "&mut Node"), ("name", "&[u8]")]),
    "cJSON_AddTrueToObject": ("Option<Node>", [("object", "&mut Node"), ("name", "&[u8]")]),
    "cJSON_AddFalseToObject": ("Option<Node>", [("object", "&mut Node"), ("name", "&[u8]")]),
    "cJSON_AddBoolToObject": ("Option<Node>", [
        ("object", "&mut Node"), ("name", "&[u8]"), ("boolean", "bool")]),
    "cJSON_AddNumberToObject": ("Option<Node>", [
        ("object", "&mut Node"), ("name", "&[u8]"), ("number", "f64")]),
    "cJSON_AddStringToObject": ("Option<Node>", [
        ("object", "&mut Node"), ("name", "&[u8]"), ("string", "&[u8]")]),
    "cJSON_AddRawToObject": ("Option<Node>", [
        ("object", "&mut Node"), ("name", "&[u8]"), ("raw", "&[u8]")]),
    "cJSON_AddObjectToObject": ("Option<Node>", [("object", "&mut Node"), ("name", "&[u8]")]),
    "cJSON_AddArrayToObject": ("Option<Node>", [("object", "&mut Node"), ("name", "&[u8]")]),
    "cJSON_DetachItemViaPointer": ("Option<Node>", [
        ("parent", "&mut Node"), ("item", "&Node")]),
    "cJSON_DetachItemFromArray": ("Option<Node>", [("array", "&mut Node"), ("which", "i32")]),
    "cJSON_DeleteItemFromArray": ("()", [("array", "&mut Node"), ("which", "i32")]),
    "cJSON_DetachItemFromObject": ("Option<Node>", [("object", "&mut Node"), ("string", "&[u8]")]),
    "cJSON_DetachItemFromObjectCaseSensitive": ("Option<Node>", [
        ("object", "&mut Node"), ("string", "&[u8]")]),
    "cJSON_DeleteItemFromObject": ("()", [("object", "&mut Node"), ("string", "&[u8]")]),
    "cJSON_DeleteItemFromObjectCaseSensitive": ("()", [
        ("object", "&mut Node"), ("string", "&[u8]")]),
    "cJSON_InsertItemInArray": ("bool", [
        ("array", "&mut Node"), ("which", "i32"), ("item", "Node")]),
    "cJSON_ReplaceItemViaPointer": ("bool", [
        ("object", "&mut Node"), ("old", "&Node"), ("new", "Node")]),
    "cJSON_ReplaceItemInArray": ("bool", [
        ("array", "&mut Node"), ("which", "i32"), ("new", "Node")]),
    "cJSON_ReplaceItemInObject": ("bool", [
        ("object", "&mut Node"), ("string", "&[u8]"), ("new", "Node")]),
    "cJSON_ReplaceItemInObjectCaseSensitive": ("bool", [
        ("object", "&mut Node"), ("string", "&[u8]"), ("new", "Node")]),
    "cJSON_CreateNull": ("Option<Node>", []),
    "cJSON_CreateTrue": ("Option<Node>", []),
    "cJSON_CreateFalse": ("Option<Node>", []),
    "cJSON_CreateBool": ("Option<Node>", [("boolean", "bool")]),
    "cJSON_CreateNumber": ("Option<Node>", [("number", "f64")]),
    "cJSON_CreateString": ("Option<Node>", [("string", "&[u8]")]),
    "cJSON_CreateStringReference": ("Option<Node>", [("string", "&[u8]")]),
    "cJSON_CreateObjectReference": ("Option<Node>", [("child", "&Node")]),
    "cJSON_CreateArrayReference": ("Option<Node>", [("child", "&Node")]),
    "cJSON_CreateRaw": ("Option<Node>", [("raw", "&[u8]")]),
    "cJSON_CreateArray": ("Option<Node>", []),
    "cJSON_CreateObject": ("Option<Node>", []),
    "cJSON_CreateIntArray": ("Option<Node>", [("numbers", "&[i32]"), ("count", "i32")]),
    "cJSON_CreateFloatArray": ("Option<Node>", [("numbers", "&[f32]"), ("count", "i32")]),
    "cJSON_CreateDoubleArray": ("Option<Node>", [("numbers", "&[f64]"), ("count", "i32")]),
    "cJSON_CreateStringArray": ("Option<Node>", [("strings", "&[&[u8]]"), ("count", "i32")]),
    "cJSON_malloc": ("Option<Vec<u8>>", [("size", "usize")]),
    "cJSON_free": ("()", [("object", "Vec<u8>")]),
}

# internal helpers: mostly mechanical, but a few need explicit shapes
INTERNAL_OVERRIDES: dict[str, tuple[str, list[tuple[str, str]]]] = {
    "case_insensitive_strcmp": ("i32", [("string1", "&[u8]"), ("string2", "&[u8]")]),
    "get_decimal_point": ("u8", []),
    "parse_number": ("bool", [("item", "&mut Node"), ("input_buffer", "&mut ParseBuffer<'_>")]),
    "ensure": ("Option<usize>", [("p", "&mut PrintBuffer"), ("needed", "usize")]),
    "update_offset": ("()", [("buffer", "&mut PrintBuffer")]),
    "compare_double": ("bool", [("a", "f64"), ("b", "f64")]),
    "print_number": ("bool", [("item", "&Node"), ("output_buffer", "&mut PrintBuffer")]),
    "parse_hex4": ("u32", [("input", "&[u8]")]),
    "utf16_literal_to_utf8": ("Option<Vec<u8>>", [
        ("input_pointer", "&[u8]"), ("input_end", "&[u8]"),
        ("output_pointer", "&mut Vec<u8>")]),
    "parse_string": ("bool", [("item", "&mut Node"), ("input_buffer", "&mut ParseBuffer<'_>")]),
    "print_string_ptr": ("bool", [
        ("input", "Option<&[u8]>"), ("output_buffer", "&mut PrintBuffer")]),
    "print_string": ("bool", [("item", "&Node"), ("p", "&mut PrintBuffer")]),
    "print": ("Option<Vec<u8>>", [("item", "&Node"), ("format", "bool"), ("hooks", "&Hooks")]),
    "buffer_skip_whitespace": ("()", [("buffer", "&mut ParseBuffer<'_>")]),
    "skip_utf8_bom": ("()", [("buffer", "&mut ParseBuffer<'_>")]),
    "parse_value": ("bool", [("item", "&mut Node"), ("input_buffer", "&mut ParseBuffer<'_>")]),
    "print_value": ("bool", [("item", "&Node"), ("output_buffer", "&mut PrintBuffer")]),
    "parse_array": ("bool", [("item", "&mut Node"), ("input_buffer", "&mut ParseBuffer<'_>")]),
    "print_array": ("bool", [("item", "&Node"), ("output_buffer", "&mut PrintBuffer")]),
    "parse_object": ("bool", [("item", "&mut Node"), ("input_buffer", "&mut ParseBuffer<'_>")]),
    "print_object": ("bool", [("item", "&Node"), ("output_buffer", "&mut PrintBuffer")]),
    "get_array_item": ("Option<&Node>", [("array", "&Node"), ("index", "usize")]),
    "get_object_item": ("Option<&'a Node>", [
        ("object", "&'a Node"), ("name", "&[u8]"), ("case_sensitive", "bool")]),
    "suffix_object": ("()", [("prev", "&mut Node"), ("item", "Node")]),
    "create_reference": ("Option<Node>", [("item", "&Node"), ("hooks", "&Hooks")]),
    "add_item_to_array": ("bool", [("array", "&mut Node"), ("item", "Node")]),
    "cast_away_const": ("&'a [u8]", [("string", "&&'a [u8]")]),
    "add_item_to_object": ("bool", [
        ("object", "&mut Node"), ("string", "&[u8]"), ("item", "Node"),
        ("hooks", "&Hooks"), ("constant_key", "bool")]),
    "replace_item_in_object": ("bool", [
        ("object", "&mut Node"), ("string", "&[u8]"), ("new_item", "Node"),
        ("case_sensitive", "bool")]),
    "cJSON_Duplicate_rec": ("Option<Node>", [
        ("item", "&Node"), ("depth", "usize"), ("recursion", "bool")]),
    "skip_oneline_comment": ("()", [("input", "&mut &[u8]")]),
    "skip_multiline_comment": ("()", [("input", "&mut &[u8]")]),
    "minify_string": ("()", [("input", "&mut &[u8]"), ("output", "&mut Vec<u8>")]),
}


def build_target_signatures() -> dict:
    cg = json.load(open(os.path.join(A, "callgraph.json")))
    out = {}
    for cname, f in cg["functions"].items():
        rn = rust_name(cname)
        if cname in OVERRIDES:
            ret, params = OVERRIDES[cname]
        elif cname in INTERNAL_OVERRIDES:
            ret, params = INTERNAL_OVERRIDES[cname]
        else:
            ret = rust_type(f["return_type"], ret=True, fname=cname)
            params = []
            for p in f["params"]:
                params.append((p["name"], rust_type(p["type"], fname=cname)))
        all_text = ' '.join([ret] + [t for _, t in params])
        lifetimes = sorted(list(set(re.findall(r"'([a-z])\b", all_text))))
        lt_args = [f"'{l}" for l in lifetimes]
        lt_str = f"<{', '.join(lt_args)}>" if lifetimes else ""
        sig = f"fn {rn}{lt_str}({', '.join(f'{n}: {t}' for n, t in params)}) -> {ret}"
        out[cname] = {
            "c_name": cname,
            "rust_name": rn,
            "c_signature": f"{f['return_type']} {cname}({', '.join(p['type'] + ' ' + p['name'] for p in f['params'])})",
            "rust_signature": sig,
            "params": [{"name": n, "rust_type": t} for n, t in params],
            "return_type": ret,
        }
    return out


def validate_signatures(sigs: dict) -> list[str]:
    """Every type named in a signature must exist in the contract.

    A signature that mentions a type the contract does not define produces a
    candidate that can never compile, so this is checked, not assumed.
    """
    src = open(os.path.join(A, "rust_contract.rs")).read()
    known = set(re.findall(r"pub (?:struct|enum|type|fn|const) ([A-Za-z_][A-Za-z0-9_]*)", src))
    known |= set(re.findall(r"pub const ([A-Za-z_][A-Za-z0-9_]*)", src))
    prelude = {"Option", "Some", "None", "Vec", "String", "bool", "usize", "i32",
               "u32", "u8", "u64", "i64", "f64", "str", "Result", "Self"}
    bad = []
    for cname, s in sigs.items():
        for t in set(re.findall(r"\b([A-Z][A-Za-z0-9_]*)\b", s["rust_signature"])):
            if t not in known and t not in prelude:
                bad.append(f"{cname}: unknown type `{t}`")
    return bad


def main() -> None:
    os.makedirs(A, exist_ok=True)
    with open(os.path.join(A, "rust_contract.rs"), "w") as fh:
        fh.write(CONTRACT_RS)
    sigs = build_target_signatures()
    bad = validate_signatures(sigs)
    if bad:
        print("SIGNATURE VALIDATION FAILED:")
        for b in bad:
            print("   " + b)
        raise SystemExit(1)
    with open(os.path.join(A, "target_signatures.json"), "w") as fh:
        json.dump(sigs, fh, indent=2)
    print(f"contract written: artifacts/analysis/rust_contract.rs")
    print(f"target signatures: {len(sigs)}")
    missing = [c for c, s in sigs.items() if s["rust_signature"].count("()") > 0]
    print("\nsample:")
    for c in ["cJSON_Parse", "parse_value", "cJSON_AddItemToObject", "ensure",
              "cJSON_CreateObjectReference", "cJSON_Minify", "print_number"]:
        if c in sigs:
            print(f"  {sigs[c]['c_signature']}")
            print(f"    -> {sigs[c]['rust_signature']}")


if __name__ == "__main__":
    main()
