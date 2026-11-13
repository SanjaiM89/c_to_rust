//@unit: U09_create_p4
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

pub fn create_string_array(strings: &[&[u8]], count: i32) -> Option<Node> {
    if count < 0 { return None; }
    let count = count as usize;
    let mut arr = create_array()?;
    for &s in strings.iter().take(count) {
        let n = create_string(s)?;
        arr.children.push(n);
    }
    Some(arr)
}

pub fn create_string_reference(string: &[u8]) -> Option<Node> {
    let mut item = Node::new(CJSON_STRING | CJSON_IS_REFERENCE);
    item.valuestring = Some(string.to_vec());
    Some(item)
}

pub fn create_true() -> Option<Node> {
    Some(Node::new(CJSON_TRUE))
}

pub fn set_valuestring(object: &mut Node, valuestring: &[u8]) -> Option<Vec<u8>> {
    if (object.type_ & CJSON_STRING) == 0 || (object.type_ & CJSON_IS_REFERENCE) != 0 {
        return None;
    }
    let copy = valuestring.to_vec();
    object.valuestring = Some(copy.clone());
    Some(copy)
}

pub fn create_reference(item: &Node, _hooks: &Hooks) -> Option<Node> {
    let mut ref_node = item.clone();
    ref_node.string = None;
    ref_node.type_ |= CJSON_IS_REFERENCE;
    Some(ref_node)
}

pub use create_string_array as cJSON_CreateStringArray;
pub use create_string_reference as cJSON_CreateStringReference;
pub use create_true as cJSON_CreateTrue;
pub use set_valuestring as cJSON_SetValuestring;
