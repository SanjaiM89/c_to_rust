pub fn get_object_item_case_sensitive<'a>(
    object: Option<&'a Node>,
    string: &[u8],
) -> Option<&'a Node> {
    let obj = object?;
    get_object_item_internal(obj, string, true)
}
