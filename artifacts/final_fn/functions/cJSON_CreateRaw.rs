pub fn create_raw(raw: &[u8]) -> Option<Node> {
    let mut item = Node::new(CJSON_RAW);
    item.valuestring = Some(raw.to_vec());
    Some(item)
}
