pub fn create_string_reference(string: &[u8]) -> Option<Node> {
    let mut item = Node::new(CJSON_STRING | CJSON_IS_REFERENCE);
    item.valuestring = Some(string.to_vec());
    Some(item)
}
