pub fn create_object_reference(child: &Node) -> Option<Node> {
    let mut item = Node::new(CJSON_OBJECT | CJSON_IS_REFERENCE);
    item.children.push(child.clone());
    Some(item)
}
