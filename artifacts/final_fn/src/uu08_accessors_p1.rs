use crate::contract::*;

pub fn get_array_item(array: Option<&Node>, index: i32) -> Option<&Node> {
    if index < 0 {
        return None;
    }
    array.as_ref().and_then(|array| {
        if index as usize >= array.children.len() {
            None
        } else {
            Some(&array.children[index as usize])
        }
    })
}

pub fn get_array_size(array: Option<&Node>) -> i32 {
    let mut size = 0;
    let mut child = array.as_ref().and_then(|n| n.children.first());
    
    while let Some(node) = child {
        size += 1;
        child = node.children.first();
    }
    
    size as i32
}

pub fn get_error_ptr() -> Option<&'static [u8]> {
    GLOBAL_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map(|(msg, _)| msg)
    })
}

pub fn get_number_value(item: Option<&Node>) -> f64 {
    if item.map(|n| n.type_ & CJSON_NUMBER != 0).unwrap_or(false) {
        item.as_ref().unwrap().valuedouble
    } else {
        f64::NAN
    }
}

pub fn is_number(item: Option<&Node>) -> bool {
    item.map(|n| n.type_ & CJSON_NUMBER != 0).unwrap_or(false)
}

pub fn get_array_item(array: &Node, index: usize) -> Option<&Node> {
    if index >= array.children.len() {
        return None;
    }

    let mut current = &array.children[0];
    let mut current_idx = 0;

    while current_idx < index && current_idx < array.children.len() {
        current_idx += 1;
        current = current.children.get(current_idx).unwrap_or(&array.children[current_idx]);
    }

    Some(current)
}
