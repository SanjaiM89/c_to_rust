pub fn detach_item_from_array(array: &mut Node, which: i32) -> Option<Node> {
    if which < 0 {
        return None;
    }
    let idx = which as usize;
    if idx >= array.children.len() {
        return None;
    }
    Some(array.children.remove(idx))
}
