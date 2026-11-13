// unit: U09_create_p3
pub fn create_null() -> Option<Node> {
    Some(Node::new(CJSON_NULL))
}

pub fn create_object() -> Option<Node> {
    Some(Node::new(CJSON_OBJECT))
}

pub fn create_object_reference(child: &Node) -> Option<Node> {
    let mut item = Node::new(CJSON_OBJECT | CJSON_IS_REFERENCE);
    item.children.push(child.clone());
    Some(item)
}

pub fn create_raw(raw: &[u8]) -> Option<Node> {
    let mut item = Node::new(CJSON_RAW);
    item.valuestring = Some(raw.to_vec());
    Some(item)
}

pub fn create_string(string: &[u8]) -> Option<Node> {
    let mut item = Node::new(CJSON_STRING);
    item.valuestring = Some(string.to_vec());
    Some(item)
}
