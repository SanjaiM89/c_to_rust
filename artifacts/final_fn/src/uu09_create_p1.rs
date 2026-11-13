use crate::contract::*;

pub fn create_array() -> Option<Node> {
    Some(Node {
        type_: CJSON_ARRAY,
        valueint: 0,
        valuedouble: 0.0,
        string: None,
        children: Vec::new(),
    })
}

pub fn create_array_reference(child: &Node) -> Option<Node> {
    let mut item = Node::new();
    item.type_ = CJSON_ARRAY | CJSON_IS_REFERENCE;
    item.children.push(child.clone());
    Some(item)
}

pub fn create_bool(boolean: bool) -> Option<Node> {
    Some(Node {
        type_: if boolean { CJSON_TRUE } else { CJSON_FALSE },
        children: Vec::new(),
        string: None,
        valuedouble: None,
        valueint: None,
    })
}

pub fn create_number(number: f64) -> Option<Node> {
    Some(Node {
        type_: CJSON_NUMBER as u32,
        valuedouble: number,
        valueint: if number >= i32::MAX as f64 {
            i32::MAX
        } else if number <= i32::MIN as f64 {
            i32::MIN
        } else {
            number as i32
        },
        string: None,
        valuestring: None,
        children: Vec::new(),
    })
}

pub fn cast_away_const(string: &&[u8]) -> &[u8] {
    string
}
