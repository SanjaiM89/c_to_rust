pub fn is_string(item: Option<&Node>) -> bool {
    item.map(|n| n.type_ & CJSON_STRING == CJSON_STRING).unwrap_or(false)
}
