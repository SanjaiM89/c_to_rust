pub fn is_number(item: Option<&Node>) -> bool {
    item.map(|n| n.type_ & CJSON_NUMBER != 0).unwrap_or(false)
}
