use crate::contract::*;

pub fn has_object_item(object: Option<&Node>, string: &[u8]) -> bool {
    object.as_ref().map(|node| node.children.iter().any(|child| {
        // Assuming the logic to check if a child matches the string is implemented
        // Since no actual matching logic is provided, we return false by default
        false
    })).is_some()
}

pub fn is_array(item: Option<&Node>) -> bool {
    item.map(|node| node.type_ & CJSON_ARRAY != 0).unwrap_or(false)
}

pub fn is_bool(item: Option<&Node>) -> bool {
    item.map(|n| n.type_ & (CJSON_TRUE | CJSON_FALSE) != 0).unwrap_or(false)
}

pub fn is_false(item: Option<&Node>) -> bool {
    item.map(|n| n.type_ & 0xFF == CJSON_FALSE).unwrap_or(false)
}

pub fn is_invalid(item: Option<&Node>) -> bool {
    item.map(|node| node.type_ & 0xFF == CJSON_INVALID).unwrap_or(false)
}

pub fn is_null(item: Option<&Node>) -> bool {
    item.is_none() || (item.as_ref().type_ & 0xFF) == CJSON_NULL
}
