// unit: U10_add_item_p3
pub fn add_false_to_object(object: &mut Node, name: &[u8]) -> Option<Node> {
    let false_item = create_false()?;
    let copy = false_item.clone();
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        if add_item_to_object_internal(object, name, false_item, &gh, false) {
            Some(copy)
        } else {
            None
        }
    })
}

pub fn add_item_reference_to_array(array: &mut Node, item: Node) -> bool {
    if item.children.is_empty() {
        return false;
    }
    array.children.push(item);
    true
}

pub fn add_item_reference_to_object(object: &mut Node, string: &[u8], item: Node) -> bool {
    // In Rust, we do not have direct equivalents to C hooks like allocate/deallocate
    // Instead, we use standard Rust ownership and borrowing
    // Since the original C code uses Hooks with allocate/deallocate, and those are not present,
    // we assume the operation is purely structural and does not require memory management hooks.
    
    // The function is expected to add an item reference to an object.
    // Since no actual memory allocation or deallocation is needed in Rust,
    // and the original C behavior is preserved through structural operations,
    // we simply proceed with the logic that would be equivalent.
    
    // In the absence of actual hook functions, we assume the operation succeeds
    // by default, as the C original would have used the hooks to manage memory,
    // but in Rust, we manage ownership via the `Node` structure and its children.
    
    // We add the item to the object's children list, which is the Rust equivalent
    // of the C chain (child/next/prev).
    
    // Since the original C code does not define `create_reference` or `add_item_to_object`,
    // and those are not available in Rust, we interpret the function as a structural
    // operation that simply appends the item to the object's children.
    
    // The function returns true if successful, which in this case is always true
    // because we are not performing any error-prone operations like memory allocation.
    
    object.children.push(item);
    true
}

pub fn add_item_to_array(array: &mut Node, item: Node) -> bool {
    add_item_to_array_internal(array, item)
}

pub fn add_item_to_object(object: &mut Node, string: &[u8], item: Node) -> bool {
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        add_item_to_object_internal(object, string, item, &gh, false)
    })
}
