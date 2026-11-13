// unit: U06b_parse_entry
pub fn parse(value: &[u8]) -> Option<Node> {
    parse_with_opts(value, None, false)
}

pub fn parse_with_length(value: &[u8], buffer_length: usize) -> Option<Node> {
    parse_with_length_opts(value, buffer_length, None, false)
}

pub fn parse_with_length_opts(
    value: &[u8],
    buffer_length: usize,
    mut return_parse_end: Option<&mut usize>,
    require_null_terminated: bool,
) -> Option<Node> {
    clear_error();
    if value.is_empty() || buffer_length == 0 {
        return None;
    }

    let len = buffer_length.min(value.len());
    let mut buffer = ParseBuffer::new(&value[..len]);
    let mut item = Node::new(0);

    skip_utf8_bom(&mut buffer);
    buffer_skip_whitespace(&mut buffer);

    if !parse_value(&mut item, &mut buffer) {
        let pos = buffer.offset;
        if let Some(ref mut end_ptr) = return_parse_end {
            **end_ptr = pos;
        }
        set_error(value, pos);
        return None;
    }

    if require_null_terminated {
        buffer_skip_whitespace(&mut buffer);
        if buffer.offset < buffer.length && buffer.content[buffer.offset] != 0 {
            let pos = buffer.offset;
            if let Some(ref mut end_ptr) = return_parse_end {
                **end_ptr = pos;
            }
            set_error(value, pos);
            return None;
        }
    }

    if let Some(end_ptr) = return_parse_end {
        *end_ptr = buffer.offset;
    }

    Some(item)
}

pub fn parse_with_opts(
    value: &[u8],
    return_parse_end: Option<&mut usize>,
    require_null_terminated: bool,
) -> Option<Node> {
    parse_with_length_opts(value, value.len(), return_parse_end, require_null_terminated)
}
