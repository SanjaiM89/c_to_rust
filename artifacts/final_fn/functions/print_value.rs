pub fn print_value(item: &Node, output_buffer: &mut PrintBuffer) -> bool {
    match item.type_ & 0xFF {
        CJSON_NULL => {
            let offset = match ensure(output_buffer, 5) {
                Some(off) => off,
                None => return false,
            };
            output_buffer.buffer[offset..offset + 4].copy_from_slice(b"null");
            output_buffer.offset = offset + 4;
            true
        }
        CJSON_FALSE => {
            let offset = match ensure(output_buffer, 6) {
                Some(off) => off,
                None => return false,
            };
            output_buffer.buffer[offset..offset + 5].copy_from_slice(b"false");
            output_buffer.offset = offset + 5;
            true
        }
        CJSON_TRUE => {
            let offset = match ensure(output_buffer, 5) {
                Some(off) => off,
                None => return false,
            };
            output_buffer.buffer[offset..offset + 4].copy_from_slice(b"true");
            output_buffer.offset = offset + 4;
            true
        }
        CJSON_NUMBER => print_number(item, output_buffer),
        CJSON_RAW => {
            let val = match item.valuestring.as_ref() {
                Some(v) => v,
                None => return false,
            };
            let offset = match ensure(output_buffer, val.len() + 1) {
                Some(off) => off,
                None => return false,
            };
            output_buffer.buffer[offset..offset + val.len()].copy_from_slice(val);
            output_buffer.offset = offset + val.len();
            true
        }
        CJSON_STRING => print_string(item, output_buffer),
        CJSON_ARRAY => print_array(item, output_buffer),
        CJSON_OBJECT => print_object(item, output_buffer),
        _ => false,
    }
}
