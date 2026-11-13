pub fn add_number_to_object(object: &mut Node, name: &[u8], number: f64) -> Option<Node> {
    let number_item = create_number(number)?;
    let copy = number_item.clone();
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        if add_item_to_object_internal(object, name, number_item, &gh, false) {
            Some(copy)
        } else {
            None
        }
    })
}
