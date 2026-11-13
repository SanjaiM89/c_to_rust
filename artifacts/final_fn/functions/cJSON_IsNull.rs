pub fn is_null(item: Option<&Node>) -> bool {
    match item {
        Some(item) => (item.type_ & 0xFF) == CJSON_NULL,
        None => false,
    }
}
