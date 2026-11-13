use crate::contract::*;

pub fn buffer_skip_whitespace(buffer: &mut ParseBuffer<'_>) -> &mut ParseBuffer<'_> {
    if buffer.offset >= buffer.length {
        return buffer;
    }

    while buffer.offset < buffer.length && buffer.content[buffer.offset] <= b' ' as u8 {
        buffer.offset += 1;
    }

    if buffer.offset == buffer.length {
        buffer.offset -= 1;
    }

    buffer
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
    let mut offset = input_buffer.offset;
    while offset < input_buffer.length && input_buffer.content[offset] == b' ' {
        offset += 1;
    }

    if offset >= input_buffer.length || input_buffer.content[offset] == b']' {
        input_buffer.depth -= 1;
        item.type_ = CJSON_ARRAY;
        return true;
    }

    if offset >= input_buffer.length {
        input_buffer.offset -= 1;
        input_buffer.depth -= 1;
        return false;
    }

    input_buffer.offset = offset;

    let mut head = None;
    let mut current_item = None;

    loop {
        let new_item = Node::new(0);
        if new_item.children.is_empty() {
            input_buffer.depth -= 1;
            return false;
        }

        let mut item = new_item;

        if head.is_none() {
            head = Some(item);
            current_item = Some(item);
        } else {
            let current = current_item.as_mut().unwrap();
            current.children.push(item);
            current_item = Some(item);
        }

        input_buffer.offset += 1;
        while input_buffer.offset < input_buffer.length && input_buffer.content[input_buffer.offset] == b' ' {
            input_buffer.offset += 1;
        }

        if !parse_value(&mut current_item.as_mut().unwrap(), input_buffer) {
            input_buffer.depth -= 1;
            return false;
        }

        input_buffer.offset += 1;
        while input_buffer.offset < input_buffer.length && input_buffer.content[input_buffer.offset] == b' ' {
            input_buffer.offset += 1;
        }

        if input_buffer.offset >= input_buffer.length || input_buffer.content[input_buffer.offset] != b',' {
            break;
        }
    }

    if input_buffer.offset >= input_buffer.length || input_buffer.content[input_buffer.offset] != b']' {
        input_buffer.depth -= 1;
        return false;
    }

    input_buffer.offset += 1;
    input_buffer.depth -= 1;

    item.type_ = CJSON_ARRAY;
    if let Some(head_node) = head {
        item.children = head_node.children;
    } else {
        item.children = Vec::new();
    }

    true
}

pub fn parse_object(item: &mut Node, input_buffer: &mut ParseBuffer<'_>) -> bool {
    if input_buffer.depth >= CJSON_NESTING_LIMIT {
        return false;
    }
    input_buffer.depth += 1;

    if input_buffer.offset >= input_buffer.length || input_buffer.content[input_buffer.offset] != b'{' {
        return false;
    }
    input_buffer.offset += 1;

    // Skip whitespace after '{'
    let mut offset = input_buffer.offset;
    while offset < input_buffer.length && input_buffer.content[offset] == b' ' {
        offset += 1;
    }
    input_buffer.offset = offset;

    if input_buffer.offset >= input_buffer.length || input_buffer.content[input_buffer.offset] == b'}' {
        return true;
    }

    if input_buffer.offset >= input_buffer.length {
        input_buffer.offset -= 1;
        return false;
    }

    input_buffer.offset -= 1;

    loop {
        // Create a new item
        let new_item = Node::new(0); // Use default type 0 as per contract

        if item.children.is_empty() {
            item.children.push(new_item);
        } else {
            let last = item.children.last().unwrap();
            item.children.push(new_item);
        }

        if input_buffer.offset >= input_buffer.length {
            return false;
        }

        input_buffer.offset += 1;

        // Skip whitespace after new item
        while input_buffer.offset < input_buffer.length && input_buffer.content[input_buffer.offset] == b' ' {
            input_buffer.offset += 1;
        }

        if input_buffer.offset >= input_buffer.length || input_buffer.content[input_buffer.offset] == b'{' {
            return false;
        }

        // Parse string key
        let last_child = item.children.last().unwrap();
        if !parse_string(&mut last_child.children.last_mut().unwrap(), input_buffer) {
            return false;
        }

        // Skip whitespace after key
        while input_buffer.offset < input_buffer.length && input_buffer.content[input_buffer.offset] == b' ' {
            input_buffer.offset += 1;
        }

        if input_buffer.offset >= input_buffer.length || input_buffer.content[input_buffer.offset] != b':' {
            return false;
        }
        input_buffer.offset += 1;

        // Parse value
        let last_child = item.children.last().unwrap();
        if !parse_value(&mut last_child.children.last_mut().unwrap(), input_buffer) {
            return false;
        }

        // Skip whitespace after value
        while input_buffer.offset < input_buffer.length && input_buffer.content[input_buffer.offset] == b' ' {
            input_buffer.offset += 1;
        }

        if input_buffer.offset >= input_buffer.length || input_buffer.content[input_buffer.offset] != b',' {
            break;
        }
        input_buffer.offset += 1;
    }

    if input_buffer.offset >= input_buffer.length || input_buffer.content[input_buffer.offset] != b'}' {
        return false;
    }

    true
}

pub fn parse_value(item: &mut Node, input_buffer: &mut ParseBuffer<'_>) -> bool {
    if input_buffer.content.is_empty() {
        return false;
    }

    let content = input_buffer.content;
    let offset = input_buffer.offset;

    // null
    if content.len() >= 4 && &content[offset..offset + 4] == b"null" {
        item.type_ = CJSON_NULL;
        input_buffer.offset += 4;
        return true;
    }

    // false
    if content.len() >= 5 && &content[offset..offset + 5] == b"false" {
        item.type_ = CJSON_FALSE;
        input_buffer.offset += 5;
        return true;
    }

    // true
    if content.len() >= 4 && &content[offset..offset + 4] == b"true" {
        item.type_ = CJSON_TRUE;
        item.valueint = 1;
        input_buffer.offset += 4;
        return true;
    }

    // string
    if !content.is_empty() && content[offset] == b'"' {
        let mut i = offset + 1;
        while i < content.len() && content[i] != b'"' {
            i += 1;
        }
        if i < content.len() && content[i] == b'"' {
            let str_data = &content[offset + 1..i];
            item.type_ = CJSON_STRING;
            item.valuestring = Some(str_data.to_vec());
            input_buffer.offset = i + 1;
            return true;
        }
    }

    // number
    if !content.is_empty() && (content[offset] == b'-' || (content[offset] >= b'0' && content[offset] <= b'9')) {
        let mut i = offset;
        while i < content.len() && (content[i] >= b'0' && content[i] <= b'9') || content[i] == b'.' || content[i] == b'-' {
            i += 1;
        }
        if i > offset {
            let num_str = &content[offset..i];
            if let Ok(num) = num_str.parse::<f64>() {
                item.type_ = CJSON_NUMBER;
                item.valuedouble = Some(num);
                input_buffer.offset = i;
                return true;
            }
        }
    }

    // array
    if !content.is_empty() && content[offset] == b'[' {
        let mut i = offset + 1;
        let mut array = Vec::new();
        while i < content.len() && content[i] != b']' {
            let mut child = Node::default();
            if !parse_value(&mut child, &mut ParseBuffer::new(&content)) {
                return false;
            }
            array.push(child);
            if i < content.len() && content[i] == b',' {
                i += 1;
            }
        }
        if i < content.len() && content[i] == b']' {
            input_buffer.offset = i + 1;
            item.type_ = CJSON_ARRAY;
            item.children = array;
            return true;
        }
    }

    // object
    if !content.is_empty() && content[offset] == b'{' {
        let mut i = offset + 1;
        let mut object = Vec::new();
        while i < content.len() && content[i] != b'}' {
            let mut key = Node::default();
            let mut value = Node::default();
            if !parse_value(&mut key, &mut ParseBuffer::new(&content)) {
                return false;
            }
            if i < content.len() && content[i] == b':' {
                i += 1;
                if !parse_value(&mut value, &mut ParseBuffer::new(&content)) {
                    return false;
                }
            } else {
                return false;
            }
            if i < content.len() && content[i] == b',' {
                i += 1;
            }
            object.push((key, value));
        }
        if i < content.len() && content[i] == b'}' {
            input_buffer.offset = i + 1;
            item.type_ = CJSON_OBJECT;
            item.children = object.into_iter().map(|(k, v)| k).collect();
            return true;
        }
    }

    false
}

pub fn skip_utf8_bom<'a>(buffer: &'a mut ParseBuffer<'a>) -> &'a mut ParseBuffer<'a> {
    if buffer.offset != 0 {
        return buffer;
    }

    if buffer.content.len() >= 4 {
        let bytes = &buffer.content[buffer.offset..];
        if bytes[..3] == *b"\xEF\xBB\xBF" {
            buffer.offset += 3;
        }
    }

    buffer
}
