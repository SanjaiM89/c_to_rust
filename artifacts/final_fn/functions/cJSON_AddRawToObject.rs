pub fn add_raw_to_object(object: &mut Node, name: &[u8], raw: &[u8]) -> Option<Node> {
    let raw_item = create_raw(raw)?;
    let copy = raw_item.clone();
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        if add_item_to_object_internal(object, name, raw_item, &gh, false) {
            Some(copy)
        } else {
            None
        }
    })
}
