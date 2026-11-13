pub fn get_array_item(array: Option<&Node>, index: i32) -> Option<&Node> {
    if index < 0 {
        return None;
    }
    let array = array?;
    get_array_item_internal(array, index as usize)
}
