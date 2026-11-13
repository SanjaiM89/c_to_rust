pub fn insert_item_in_array(array: &mut Node, which: i32, item: Node) -> bool {
    if which < 0 || item.children.is_empty() {
        return false;
    }

    let target_idx = which as usize;
    if target_idx >= array.children.len() {
        return false;
    }

    let target_node = &array.children[target_idx];
    if target_node.children.is_empty() || target_node.children.first().is_none() {
        return false;
    }

    let mut new_children = array.children.clone();
    let target_idx = new_children.iter().position(|n| n == target_node).unwrap();

    new_children.insert(target_idx + 1, item);

    array.children = new_children;
    true
}
