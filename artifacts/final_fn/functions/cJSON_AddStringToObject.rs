pub fn add_string_to_object(object: &mut Node, name: &[u8], string: &[u8]) -> Option<Node> {
    let string_item = create_string(string)?;
    let copy = string_item.clone();
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        if add_item_to_object_internal(object, name, string_item, &gh, false) {
            Some(copy)
        } else {
            None
        }
    })
}
