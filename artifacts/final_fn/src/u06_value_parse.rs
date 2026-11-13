//@unit: U06_value_parse
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

pub fn buffer_skip_whitespace(buffer: &mut ParseBuffer<'_>) {
    while buffer.offset < buffer.length && buffer.content[buffer.offset] <= 32 {
        buffer.offset += 1;
    }
    if buffer.offset >= buffer.length {
        buffer.offset = buffer.length;
    }
}

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

pub fn parse_object(item: &mut Node, input_buffer: &mut ParseBuffer<'_>) -> bool {
    if input_buffer.depth >= CJSON_NESTING_LIMIT {
        return false;
    }
    input_buffer.depth += 1;

    if input_buffer.offset >= input_buffer.length || input_buffer.content[input_buffer.offset] != b'{' {
        input_buffer.depth -= 1;
        return false;
    }

    input_buffer.offset += 1;
    buffer_skip_whitespace(input_buffer);

    if input_buffer.offset < input_buffer.length && input_buffer.content[input_buffer.offset] == b'}' {
        input_buffer.depth -= 1;
        item.type_ = CJSON_OBJECT;
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
        if !parse_string(&mut new_item, input_buffer) {
            input_buffer.depth -= 1;
            return false;
        }
        buffer_skip_whitespace(input_buffer);

        // swap valuestring into string (key name)
        new_item.string = new_item.valuestring.take();

        if input_buffer.offset >= input_buffer.length || input_buffer.content[input_buffer.offset] != b':' {
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

    if input_buffer.offset >= input_buffer.length || input_buffer.content[input_buffer.offset] != b'}' {
        input_buffer.depth -= 1;
        return false;
    }

    input_buffer.depth -= 1;
    item.type_ = CJSON_OBJECT;
    item.children = children;
    input_buffer.offset += 1;
    true
}

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

pub fn skip_utf8_bom(buffer: &mut ParseBuffer<'_>) {
    if buffer.offset + 3 <= buffer.length && &buffer.content[buffer.offset..buffer.offset + 3] == b"\xEF\xBB\xBF" {
        buffer.offset += 3;
    }
}

