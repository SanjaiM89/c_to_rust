pub fn add_object_to_object(object: &mut Node, name: &[u8]) -> Option<Node> {
    let obj = create_object()?;
    let copy = obj.clone();
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        if add_item_to_object_internal(object, name, obj, &gh, false) {
            Some(copy)
        } else {
            None
        }
    })
}
