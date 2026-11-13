pub fn add_item_to_array_internal(array: &mut Node, item: Node) -> bool {
    array.children.push(item);
    true
}
