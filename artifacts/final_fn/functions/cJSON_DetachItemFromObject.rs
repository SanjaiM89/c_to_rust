pub fn detach_item_from_object(object: &mut Node, string: &[u8]) -> Option<Node> {
    let to_detach = get_object_item(Some(&*object), string)?.clone();
    detach_item_via_pointer(object, &to_detach)
}
