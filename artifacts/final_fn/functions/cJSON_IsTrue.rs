pub fn is_true(item: Option<&Node>) -> bool {
    item.map(|n| n.type_ & CJSON_TRUE != 0).unwrap_or(false)
}
