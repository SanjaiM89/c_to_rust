pub fn get_array_item_internal(array: &Node, index: usize) -> Option<&Node> {
    array.children.get(index)
}
