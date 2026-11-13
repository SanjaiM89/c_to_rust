pub fn create_array_reference(child: &Node) -> Option<Node> {
    let mut item = Node::new(CJSON_ARRAY | CJSON_IS_REFERENCE);
    item.children.push(child.clone());
    Some(item)
}
