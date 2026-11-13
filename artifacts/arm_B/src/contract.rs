//! Shared substrate for the cJSON -> Rust port.
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
    static GLOBAL_HOOKS: std::cell::RefCell<Hooks> =
        const { std::cell::RefCell::new(Hooks { allocate: None, deallocate: None, reallocate: None }) };
}

/// Install custom allocator hooks, or restore the defaults with `None`.
///
/// Mirrors `cJSON_InitHooks`.
pub fn init_hooks(hooks: Option<&Hooks>) {
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

/// The most recent parse error message, if any. Mirrors `cJSON_GetErrorPtr`.
pub fn get_error_ptr() -> Option<&'static [u8]> {
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
