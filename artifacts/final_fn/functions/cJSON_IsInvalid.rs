pub fn is_invalid(item: Option<&Node>) -> bool {
    item.map(|node| node.type_ & 0xFF == CJSON_INVALID).unwrap_or(false)
}
