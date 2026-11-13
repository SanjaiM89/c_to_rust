pub fn delete_item_from_array(array: &mut Node, which: i32) {
    let index = which as usize;
    if index >= array.children.len() {
        return;
    }

    array.children.remove(index);
}
