pub fn set_valuestring(object: &mut Node, valuestring: &[u8]) -> Option<Vec<u8>> {
    if (object.type_ & CJSON_STRING) == 0 || (object.type_ & CJSON_IS_REFERENCE) != 0 {
        return None;
    }
    let copy = valuestring.to_vec();
    object.valuestring = Some(copy.clone());
    Some(copy)
}
