pub fn replace_item_in_object(
    object: &mut Node,
    string: &[u8],
    new: Node,
) -> bool {
    replace_item_in_object_internal(object, string, new, false)
}
