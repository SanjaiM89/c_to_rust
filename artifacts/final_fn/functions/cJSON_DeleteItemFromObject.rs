pub fn delete_item_from_object(object: &mut Node, string: &[u8]) {
    let mut children = object.children.clone();
    let mut index = 0;
    while index < children.len() {
        if let Some(child_string) = &children[index].string {
            if child_string.as_slice() == string {
                children.remove(index);
                break;
            }
        }
        index += 1;
    }
    object.children = children;
}
