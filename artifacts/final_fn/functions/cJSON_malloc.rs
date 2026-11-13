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
