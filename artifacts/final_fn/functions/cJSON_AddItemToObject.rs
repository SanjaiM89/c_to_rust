pub fn add_item_to_object(object: &mut Node, string: &[u8], item: Node) -> bool {
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        add_item_to_object_internal(object, string, item, &gh, false)
    })
}
