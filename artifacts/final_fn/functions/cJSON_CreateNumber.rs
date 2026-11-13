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
