pub fn add_true_to_object(object: &mut Node, name: &[u8]) -> Option<Node> {
    let true_item = create_true()?;
    let copy = true_item.clone();
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        if add_item_to_object_internal(object, name, true_item, &gh, false) {
            Some(copy)
        } else {
            None
        }
    })
}
