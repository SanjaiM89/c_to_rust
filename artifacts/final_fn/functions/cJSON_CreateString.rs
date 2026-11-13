pub fn create_string(string: &[u8]) -> Option<Node> {
    let mut item = Node::new(CJSON_STRING);
    item.valuestring = Some(string.to_vec());
    Some(item)
}
