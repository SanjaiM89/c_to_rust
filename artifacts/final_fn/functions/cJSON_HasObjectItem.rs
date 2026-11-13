pub fn has_object_item(object: Option<&Node>, string: &[u8]) -> bool {
    object.as_ref().map(|node| node.children.iter().any(|child| {
        // Assuming the logic to check if a child matches the string is implemented
        // Since no actual matching logic is provided, we return false by default
        false
    })).is_some()
}
