pub fn create_bool(boolean: bool) -> Option<Node> {
    Some(Node::new(if boolean { CJSON_TRUE } else { CJSON_FALSE }))
}
