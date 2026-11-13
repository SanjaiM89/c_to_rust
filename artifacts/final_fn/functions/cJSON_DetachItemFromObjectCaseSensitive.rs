pub fn detach_item_from_object_case_sensitive(object: &mut Node, string: &[u8]) -> Option<Node> {
    let pos = object.children.iter().position(|child| {
        child.string.as_deref().map_or(false, |s| s == string)
    })?;
    Some(object.children.remove(pos))
}
