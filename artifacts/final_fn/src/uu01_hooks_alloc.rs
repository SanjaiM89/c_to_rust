use crate::contract::*;

pub fn delete(item: Node) {
    let mut current = Some(item);
    while let Some(node) = current {
        let next = node.children.iter().next().copied();
        if !node.type_ & CJSON_IS_REFERENCE != 0 && !node.children.is_empty() {
            for child in &node.children {
                delete(child.clone());
            }
        }
        if !node.type_ & CJSON_IS_REFERENCE != 0 && node.valuestring.is_some() && !node.valuestring.as_ref().unwrap().is_empty() {
            node.valuestring = Some(Vec::new());
        }
        if !node.type_ & CJSON_STRING_IS_CONST != 0 && node.string.is_some() && !node.string.as_ref().unwrap().is_empty() {
            node.string = Some(Vec::new());
        }
        current = next;
    }
}

pub fn init_hooks(hooks: Option<&Hooks>) {
    GLOBAL_HOOKS.with(|h| {
        *h.borrow_mut() = match hooks {
            Some(hooks) => {
                let mut new_hooks = Hooks::default();
                new_hooks.allocate = Some(|size| {
                    if let Some(alloc) = hooks.allocate.as_ref() {
                        alloc(size)
                    } else {
                        Some(vec![0u8; size])
                    }
                });
                new_hooks.deallocate = Some(|buffer| {
                    if let Some(dealloc) = hooks.deallocate.as_ref() {
                        dealloc(buffer)
                    } else {
                        None
                    }
                });
                new_hooks.reallocate = if let Some(alloc) = hooks.allocate.as_ref() &&
                                       let Some(dealloc) = hooks.deallocate.as_ref() {
                    Some(|buffer, size| {
                        if let Some(realloc) = hooks.reallocate.as_ref() {
                            realloc(buffer, size)
                        } else {
                            let mut new_buffer = vec![0u8; size];
                            let n = buffer.len().min(size);
                            new_buffer[..n].copy_from_slice(&buffer[..n]);
                            Some(new_buffer)
                        }
                    })
                } else {
                    None
                };
                new_hooks
            }
            None => Hooks::default(),
        };
    });
}

pub fn new__item(hooks: &Hooks) -> Option<Node> {
    let node = hooks.alloc(std::mem::size_of::<Node>());
    if node.is_some() {
        Some(Node {
            type_: 0,
            valueint: 0,
            valuedouble: 0.0,
            string: None,
            children: Vec::new(),
        })
    } else {
        None
    }
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

    let length = string.len() + 1;
    let mut copy = hooks.alloc(length)?;
    copy[..length].copy_from_slice(string);
    Some(copy)
}
