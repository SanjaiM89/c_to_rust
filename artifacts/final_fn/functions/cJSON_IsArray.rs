pub fn is_array(item: Option<&Node>) -> bool {
    item.map(|node| node.type_ & CJSON_ARRAY != 0).unwrap_or(false)
}
