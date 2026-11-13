pub fn add_bool_to_object(object: &mut Node, name: &[u8], boolean: bool) -> Option<Node> {
    let bool_item = create_bool(boolean)?;
    let copy = bool_item.clone();
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        if add_item_to_object_internal(object, name, bool_item, &gh, false) {
            Some(copy)
        } else {
            None
        }
    })
}
