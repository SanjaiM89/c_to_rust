// unit: U10_add_item_p5
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
