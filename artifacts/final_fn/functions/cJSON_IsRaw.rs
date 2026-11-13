pub fn is_raw(item: Option<&Node>) -> bool {
    item.map(|node| node.type_ & CJSON_RAW == CJSON_RAW).unwrap_or(false)
}
