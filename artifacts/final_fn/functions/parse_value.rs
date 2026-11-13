pub fn parse_value(item: &mut Node, input_buffer: &mut ParseBuffer<'_>) -> bool {
    let rem = match input_buffer.content.get(input_buffer.offset..) {
        Some(s) => s,
        None => return false,
    };
    if rem.is_empty() {
        return false;
    }

    if rem.starts_with(b"null") {
        item.type_ = CJSON_NULL;
        input_buffer.offset += 4;
        return true;
    }
    if rem.starts_with(b"false") {
        item.type_ = CJSON_FALSE;
        input_buffer.offset += 5;
        return true;
    }
    if rem.starts_with(b"true") {
        item.type_ = CJSON_TRUE;
        item.valueint = 1;
        input_buffer.offset += 4;
        return true;
    }
    let first = rem[0];
    if first == b'"' {
        return parse_string(item, input_buffer);
    }
    if first == b'-' || (b'0'..=b'9').contains(&first) {
        return parse_number(item, input_buffer);
    }
    if first == b'[' {
        return parse_array(item, input_buffer);
    }
    if first == b'{' {
        return parse_object(item, input_buffer);
    }
    false
}
