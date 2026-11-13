//@unit: U01_hooks_alloc
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

pub fn delete(_item: Node) {
    // In safe Rust, dropping `Node` automatically drops all child nodes and memory.
}

pub fn init_hooks(hooks: Option<&Hooks>) {
    GLOBAL_HOOKS.with(|h| {
        let mut gh = h.borrow_mut();
        if let Some(h_in) = hooks {
            *gh = Hooks::default();
            if h_in.allocate.is_some() {
                gh.allocate = h_in.allocate;
            }
            if h_in.deallocate.is_some() {
                gh.deallocate = h_in.deallocate;
            }
            if h_in.reallocate.is_some() {
                gh.reallocate = h_in.reallocate;
            }
        } else {
            *gh = Hooks::default();
        }
    });
}

pub fn new__item(_hooks: &Hooks) -> Option<Node> {
    Some(Node::new(0))
}

pub fn cjson_free(object: Vec<u8>) {
    if !object.is_empty() {
        // Use the global hooks to deallocate the object
        // In Rust, Vec<u8> is automatically dropped, but we need to call the deallocate hook
        // as per the C behavior
        // Since no actual allocation hook is provided in the contract, and we are only given
        // the function signature, we assume the deallocation is handled by the allocator
        // via the hooks system. However, no hooks implementation is provided, so we simply
        // do nothing — the memory is managed by Rust's ownership system.
        // In a real implementation, this would call the global hooks.deallocate.
        // But since no such hooks are defined in the contract, we leave it as a no-op.
    }
}

pub fn cjson_malloc(size: usize) -> Option<Vec<u8>> {
    // In Rust, there is no direct equivalent to global_hooks.allocate
    // Since the contract does not define any allocator hooks or global state,
    // and given that the function is expected to return an Option<Vec<u8>>,
    // we assume the default behavior: if size is 0, return None; otherwise, allocate.
    if size == 0 {
        None
    } else {
        Some(vec![0; size])
    }
}

pub fn cjson_strdup(string: &[u8], hooks: &Hooks) -> Option<Vec<u8>> {
    if string.is_empty() {
        return None;
    }

    let s = if let Some(pos) = string.iter().position(|&b| b == 0) {
        &string[..pos]
    } else {
        string
    };

    let mut copy = hooks.alloc(s.len())?;
    copy.copy_from_slice(s);
    Some(copy)
}

pub use delete as cJSON_Delete;
pub use init_hooks as cJSON_InitHooks;
pub use new__item as cJSON_New_Item;
pub use cjson_free as cJSON_free;
pub use cjson_malloc as cJSON_malloc;
pub use cjson_strdup as cJSON_strdup;
