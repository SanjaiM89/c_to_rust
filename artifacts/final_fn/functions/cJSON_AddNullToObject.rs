pub fn add_null_to_object(object: &mut Node, name: &[u8]) -> Option<Node> {
    let null_item = create_null()?;
    let copy = null_item.clone();
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        if add_item_to_object_internal(object, name, null_item, &gh, false) {
            Some(copy)
        } else {
            None
        }
    })
}
