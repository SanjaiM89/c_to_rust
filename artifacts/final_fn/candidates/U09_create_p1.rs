// unit: U09_create_p1
pub fn create_array() -> Option<Node> {
    Some(Node::new(CJSON_ARRAY))
}

pub fn create_array_reference(child: &Node) -> Option<Node> {
    let mut item = Node::new(CJSON_ARRAY | CJSON_IS_REFERENCE);
    item.children.push(child.clone());
    Some(item)
}

pub fn create_bool(boolean: bool) -> Option<Node> {
    Some(Node::new(if boolean { CJSON_TRUE } else { CJSON_FALSE }))
}

pub fn create_number(number: f64) -> Option<Node> {
    let mut item = Node::new(CJSON_NUMBER);
    item.valuedouble = number;
    if number >= i32::MAX as f64 {
        item.valueint = i32::MAX;
    } else if number <= i32::MIN as f64 {
        item.valueint = i32::MIN;
    } else {
        item.valueint = number as i32;
    }
    Some(item)
}

pub fn cast_away_const<'a>(string: &&'a [u8]) -> &'a [u8] {
    *string
}
