pub fn is_false(item: Option<&Node>) -> bool {
    item.map(|n| n.type_ & 0xFF == CJSON_FALSE).unwrap_or(false)
}
