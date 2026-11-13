pub fn add_array_to_object(object: &mut Node, name: &[u8]) -> Option<Node> {
    let array = create_array()?;
    let copy = array.clone();
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        if add_item_to_object_internal(object, name, array, &gh, false) {
            Some(copy)
        } else {
            None
        }
    })
}
