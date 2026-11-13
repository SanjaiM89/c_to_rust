use crate::contract::*;

pub fn add_item_to_array(array: &mut Node, item: Node) -> bool {
    if array.children.is_empty() || item.children.is_empty() || array.children.is_empty() {
        return false;
    }

    let child = array.children.first();
    if child.is_none() {
        array.children.push(item);
    } else {
        let last_child = array.children.last().unwrap();
        array.children.push(item);
    }

    true
}

pub fn add_item_to_object(object: &mut Node, string: &[u8], item: Node, hooks: &Hooks, constant_key: bool) -> bool {
    if object.children.is_empty() || string.is_empty() || item.children.is_empty() || *object == item {
        return false;
    }

    let new_key = if constant_key {
        string
    } else {
        if let Some(key) = hooks.alloc(string.len() + 1) {
            key[..string.len() + 1].copy_from_slice(string);
            Some(key)
        } else {
            return false;
        }
    };

    let new_type = if constant_key {
        item.type_ | CJSON_STRING_IS_CONST
    } else {
        item.type_ & !CJSON_STRING_IS_CONST
    };

    if item.type_ & CJSON_STRING_IS_CONST == 0 && !item.string.is_empty() {
        // In Rust, we skip explicit deallocation as per the contract's memory model.
        // The string is managed by the allocator and no direct deallocation is needed.
    }

    item.string = new_key;
    item.type_ = new_type;

    false
}
