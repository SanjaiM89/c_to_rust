pub fn replace_item_in_array(array: &mut Node, which: i32, new: Node) -> bool {
    if which < 0 {
        return false;
    }

    let index = which as usize;
    if index < array.children.len() {
        array.children[index] = new;
        return true;
    }

    false
}
