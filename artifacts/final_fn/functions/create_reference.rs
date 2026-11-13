pub fn create_reference(item: &Node, _hooks: &Hooks) -> Option<Node> {
    let mut ref_node = item.clone();
    ref_node.string = None;
    ref_node.type_ |= CJSON_IS_REFERENCE;
    Some(ref_node)
}
