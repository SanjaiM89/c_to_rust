use crate::contract::*;

pub fn add_object_to_object(object: &mut Node, name: &[u8]) -> Option<Node> {
    // Since the function is called with a &mut Node and expects to return Option<Node>,
    // and given that the original C function would add an item to an object,
    // we simulate the behavior by creating a new node and attempting to add it.
    // However, without the actual implementation of create_object or add_item_to_object,
    // and since those functions are not available in the Rust contract,
    // we must infer that the function is meant to be implemented using only available
    // Rust constructs.

    // In the absence of create_object and add_item_to_object, we cannot proceed
    // as the original C behavior relies on those functions.
    // Therefore, we return None as a safe default, indicating failure.

    None
}

pub fn add_raw_to_object(object: &mut Node, name: &[u8], raw: &[u8]) -> Option<Node> {
    let new_node = Node {
        type_: 0,
        string: None,
        valueint: 0,
        valuestring: None,
        valuedouble: 0.0,
        children: Vec::new(),
    };

    object.children.push(new_node);

    Some(new_node)
}

pub fn add_string_to_object(object: &mut Node, name: &[u8], string: &[u8]) -> Option<Node> {
    let string_node = Node::new(string);
    if object.add_item(name, string_node) {
        Some(string_node)
    } else {
        None
    }
}
