// unit: U10_add_item_p4
pub fn add_item_to_object_cs(object: &mut Node, string: &[u8], item: Node) -> bool {
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        add_item_to_object_internal(object, string, item, &gh, true)
    })
}

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
