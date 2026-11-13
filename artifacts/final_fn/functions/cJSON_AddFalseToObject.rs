pub fn add_false_to_object(object: &mut Node, name: &[u8]) -> Option<Node> {
    let false_item = create_false()?;
    let copy = false_item.clone();
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        if add_item_to_object_internal(object, name, false_item, &gh, false) {
            Some(copy)
        } else {
            None
        }
    })
}
