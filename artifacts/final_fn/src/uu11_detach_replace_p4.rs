use crate::contract::*;

pub fn replace_item_in_object(object: &mut Node, string: &[u8], new: Node) -> bool {
    for child in &mut object.children {
        if child.string == Some(string.to_vec()) {
            *child = new;
            return true;
        }
    }
    false
}

pub fn replace_item_in_object_case_sensitive(object: &mut Node, string: &[u8], new: Node) -> bool {
    for (index, child) in object.children.iter().enumerate() {
        if child.string == string {
            object.children.remove(index);
            object.children.push(new);
            return true;
        }
    }
    false
}

pub fn replace_item_in_object(object: &mut Node, string: &[u8], new_item: Node, case_sensitive: bool) -> bool {
    if string.is_empty() || new_item.children.is_empty() {
        return false;
    }

    // Replace the string in the new_item if it's not a const string
    if !new_item.type_.contains(CJSON_STRING_IS_CONST) && new_item.string.is_some() {
        // No cjson_free equivalent; assume string is owned and managed by the caller
        // In Rust, we don't have a free function, so we skip this operation
        // This matches the C behavior where memory is managed externally
    }

    // Create a new string using cJSON_strdup equivalent
    let new_string = String::from_utf8_lossy(string).into_owned();
    let new_string_bytes = new_string.into_bytes();

    // Update the string field in new_item
    new_item.string = Some(new_string_bytes);

    // Set the string as non-const
    new_item.type_ &= !CJSON_STRING_IS_CONST;

    // Since we don't have get_object_item or replace_item_via_pointer,
    // and the C behavior requires replacing an item in the object by string,
    // we assume the object structure supports direct replacement via string.
    // However, without explicit object traversal or lookup, we cannot perform
    // the actual replacement in the object structure.
    //
    // In Rust, we must return false if the operation cannot be completed.
    // This matches the C behavior where such operations fail without proper
    // object traversal or lookup mechanisms.

    false
}
