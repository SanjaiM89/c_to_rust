pub fn get_object_item_internal<'a>(
    object: &'a Node,
    name: &[u8],
    case_sensitive: bool,
) -> Option<&'a Node> {
    for child in &object.children {
        if let Some(s) = &child.string {
            if case_sensitive {
                if s.as_slice() == name {
                    return Some(child);
                }
            } else {
                if case_insensitive_strcmp(name, s.as_slice()) == 0 {
                    return Some(child);
                }
            }
        }
    }
    None
}
