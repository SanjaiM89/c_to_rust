use crate::contract::*;

pub fn create_null() -> Option<Node> {
    Some(Node {
        type_: CJSON_NULL,
        children: Vec::new(),
        string: None,
        valuedouble: None,
        valueint: None,
    })
}

pub fn create_object() -> Option<Node> {
    None
}

pub fn create_object_reference(child: &Node) -> Option<Node> {
    let item = None;
    if let Some(item) = item {
        item.type_ = CJSON_OBJECT | CJSON_IS_REFERENCE;
        item.children = vec![child.clone()];
    }
    item
}

pub fn create_raw(raw: &[u8]) -> Option<Node> {
    let mut item = Node {
        type_: CJSON_RAW,
        valuestring: None,
        children: Vec::new(),
        string: None,
        valuedouble: None,
        valueint: None,
    };
    item.valuestring = Some(raw.to_vec());
    Some(item)
}

pub fn create_string(string: &[u8]) -> Option<Node> {
    let mut node = Node::new();
    node.type_ = CJSON_STRING;
    if let Some(valuestring) = string.to_vec() {
        node.valuestring = Some(valuestring);
    } else {
        return None;
    }
    Some(node)
}
