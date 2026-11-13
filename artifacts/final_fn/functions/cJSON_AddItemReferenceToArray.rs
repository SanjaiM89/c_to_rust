pub fn add_item_reference_to_array(array: &mut Node, item: Node) -> bool {
    if item.children.is_empty() {
        return false;
    }
    array.children.push(item);
    true
}
