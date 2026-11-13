//! Shared substrate for the cJSON -> Rust port.
//!
//! This file is the *symbolic* part of the pipeline: it is hand-written and is
//! given unchanged to every translation unit, so that independently generated
//! units compose. The model only ever writes function bodies.

// ---------------------------------------------------------------------------
// type flags (bit values are identical to the C header)
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

/// A JSON node.
///
/// `value` holds the payload of a string or raw node and `string` holds an
/// object's key. Both are raw bytes, exactly like the C `char *` they replace:
/// cJSON preserves invalid UTF-8, so `String`/`&str` would be wrong.
//@BLOCK:node
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Node {
    /// cJSON type bitmask (`cJSON_NUMBER`, `cJSON_OBJECT`, ...).
    pub kind: u32,
    /// Payload of a string or raw node. `None` for every other kind.
    pub value: Option<Vec<u8>>,
    /// Integer view of a number node (C truncates towards zero).
    pub valueint: i32,
    /// Floating point view of a number node.
    pub valuedouble: f64,
    /// Object key (raw bytes). `None` for array elements.
    pub string: Option<Vec<u8>>,
    /// Child list. Replaces the C `child`/`next`/`prev` chain.
    pub children: Vec<Node>,
}

impl Node {
    /// Build a node of the given kind with no payload.
    pub fn new(kind: u32) -> Node {
        Node { kind, ..Default::default() }
    }

    /// Iterate over the children of this node.
    pub fn children(&self) -> std::slice::Iter<'_, Node> {
        self.children.iter()
    }
}

/// Parsing cursor over the input bytes.
///
/// Mirrors the C `parse_buffer`: `content` is the whole input, `offset` is the
/// current read position and `depth` tracks recursion depth.
//@BLOCK:parsebuf
#[derive(Debug, Clone)]
pub struct ParseBuffer<'a> {
    pub content: &'a [u8],
    pub length: usize,
    pub offset: usize,
    pub depth: usize,
}

/// Output buffer used by the printer.
///
/// Mirrors the C `printbuffer`. The content lives in `buffer[0..offset]` and
/// is NUL terminated at `buffer[offset]` (the C code relies on that in
/// `update_offset`). `length` is the allocated capacity (the C field has the
/// same name), `noalloc` forbids growing (the `cJSON_PrintPreallocated` mode).
/// The C `hooks` field has no counterpart: Rust's `Vec` grows on its own.
//@BLOCK:printbuf
#[derive(Debug, Clone, Default)]
pub struct PrintBuffer {
    pub buffer: Vec<u8>,
    pub length: usize,
    pub offset: usize,
    pub depth: usize,
    pub noalloc: bool,
    pub format: bool,
}

/// Allocator hook.
///
/// Returning `None` simulates an allocation failure. This mirrors the C
/// `cJSON_Hooks` mechanism, which the upstream test suite exercises by
/// installing an always-failing allocator.
//@BLOCK:hooks
pub type AllocHook = fn(size: usize) -> Option<Vec<u8>>;

thread_local! {
    /// The installed allocator hook. `None` means "always succeeds".
    static ALLOC_HOOK: std::cell::RefCell<Option<AllocHook>> =
        const { std::cell::RefCell::new(None) };
}

//@BLOCK:init_hooks
/// Install a custom allocator hook, or restore the default with `None`.
///
/// Mirrors `cJSON_InitHooks`.
pub fn init_hooks(hook: Option<AllocHook>) {
    ALLOC_HOOK.with(|h| {
        *h.borrow_mut() = hook;
    });
}

//@BLOCK:try_alloc
/// Ask the active hook whether an allocation of `size` bytes may succeed.
///
/// Returns `true` when the allocation may proceed.
pub fn try_alloc(size: usize) -> bool {
    ALLOC_HOOK.with(|h| {
        match *h.borrow() {
            Some(f) => f(size).is_some(),
            None => true,
        }
    })
}

//@BLOCK:error_api
thread_local! {
    /// The most recent parse error, mirroring the C `global_error` struct.
    static GLOBAL_ERROR: std::cell::RefCell<Option<(Vec<u8>, usize)>> =
        const { std::cell::RefCell::new(None) };
}

//@BLOCK:error_api
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
///
/// Mirrors `cJSON_GetErrorPtr`.
pub fn get_error_ptr() -> Option<&'static [u8]> {
    GLOBAL_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map(|(msg, _)| unsafe {
                // SAFETY: the error message lives in a thread_local that is
                // never freed while the thread is alive, and the returned
                // slice only reads immutable bytes.
                std::slice::from_raw_parts(msg.as_ptr(), msg.len())
            })
    })
}

/// The offset at which the most recent parse error occurred.
pub fn get_error_position() -> Option<usize> {
    GLOBAL_ERROR.with(|e| e.borrow().as_ref().map(|(_, pos)| *pos))
}
