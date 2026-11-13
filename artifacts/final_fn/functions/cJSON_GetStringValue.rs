pub fn get_string_value(item: Option<&Node>) -> Option<&[u8]> {
    let item = item?;
    if !is_string(Some(item)) {
        return None;
    }
    item.valuestring.as_deref()
}
