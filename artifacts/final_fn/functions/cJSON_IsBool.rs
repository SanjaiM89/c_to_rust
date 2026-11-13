pub fn is_bool(item: Option<&Node>) -> bool {
    item.map(|n| n.type_ & (CJSON_TRUE | CJSON_FALSE) != 0).unwrap_or(false)
}
