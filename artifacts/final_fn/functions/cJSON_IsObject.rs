pub fn is_object(item: Option<&Node>) -> bool {
    item.map(|node| node.type_ & CJSON_OBJECT != 0).unwrap_or(false)
}
