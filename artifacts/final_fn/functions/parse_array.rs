pub fn parse_array(item: &mut Node, input_buffer: &mut ParseBuffer<'_>) -> bool {
    if input_buffer.depth >= CJSON_NESTING_LIMIT {
        return false;
    }
    input_buffer.depth += 1;

    if input_buffer.offset >= input_buffer.length || input_buffer.content[input_buffer.offset] != b'[' {
        input_buffer.depth -= 1;
        return false;
    }

    input_buffer.offset += 1;
    buffer_skip_whitespace(input_buffer);

    if input_buffer.offset < input_buffer.length && input_buffer.content[input_buffer.offset] == b']' {
        input_buffer.depth -= 1;
        item.type_ = CJSON_ARRAY;
        input_buffer.offset += 1;
        return true;
    }

    if input_buffer.offset >= input_buffer.length {
        input_buffer.offset -= 1;
        input_buffer.depth -= 1;
        return false;
    }

    input_buffer.offset -= 1;
    let mut children = Vec::new();

    loop {
        let mut new_item = Node::new(0);
        if input_buffer.offset + 1 >= input_buffer.length {
            input_buffer.depth -= 1;
            return false;
        }

        input_buffer.offset += 1;
        buffer_skip_whitespace(input_buffer);
        if !parse_value(&mut new_item, input_buffer) {
            input_buffer.depth -= 1;
            return false;
        }
        buffer_skip_whitespace(input_buffer);
        children.push(new_item);

        if input_buffer.offset >= input_buffer.length || input_buffer.content[input_buffer.offset] != b',' {
            break;
        }
    }

    if input_buffer.offset >= input_buffer.length || input_buffer.content[input_buffer.offset] != b']' {
        input_buffer.depth -= 1;
        return false;
    }

    input_buffer.depth -= 1;
    item.type_ = CJSON_ARRAY;
    item.children = children;
    input_buffer.offset += 1;
    true
}
