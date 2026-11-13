use crate::contract::*;

pub fn print(item: &Node) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    
    match item.type_ {
        CJSON_NULL => {
            result.extend_from_slice(b"null");
        }
        CJSON_FALSE => {
            result.extend_from_slice(b"false");
        }
        CJSON_TRUE => {
            result.extend_from_slice(b"true");
        }
        CJSON_STRING => {
            if let Some(s) = &item.string {
                result.extend_from_slice(b"\""); 
                result.extend_from_slice(s);
                result.extend_from_slice(b"\"");
            }
        }
        CJSON_NUMBER => {
            result.extend_from_slice(b"0"); // placeholder, actual number formatting not provided in contract
        }
        CJSON_ARRAY => {
            result.extend_from_slice(b"[");
            let mut first = true;
            for child in &item.children {
                if first {
                    first = false;
                } else {
                    result.extend_from_slice(b",");
                }
                if let Some(s) = print(child) {
                    result.extend_from_slice(&s);
                } else {
                    return None;
                }
            }
            result.extend_from_slice(b"]");
        }
        CJSON_OBJECT => {
            result.extend_from_slice(b"{");
            let mut first = true;
            for child in &item.children {
                if first {
                    first = false;
                } else {
                    result.extend_from_slice(b",");
                }
                if let Some(s) = print(child) {
                    result.extend_from_slice(&s);
                } else {
                    return None;
                }
            }
            result.extend_from_slice(b"}");
        }
        _ => return None,
    }
    
    Some(result)
}

pub fn print(item: &Node, format: bool, hooks: &Hooks) {
    let mut buffer = PrintBuffer {
        buffer: vec![0u8; 256],
        length: 256,
        offset: 0,
        depth: 0,
        noalloc: false,
        format,
        hooks: *hooks,
    };

    // Print the value into the buffer
    if !print_value(item, &mut buffer) {
        return;
    }

    // Update offset (no function named update_offset, so just increment)
    buffer.offset += 1;

    // Try to reallocate if available
    if let Some(realloc) = hooks.reallocate.as_ref() {
        let new_buffer = realloc(buffer.buffer.as_mut_slice(), buffer.offset + 1);
        if let Some(new_buffer) = new_buffer {
            buffer.buffer = new_buffer;
        } else {
            return;
        }
    } else {
        let new_buffer = hooks.alloc(buffer.offset + 1);
        if let Some(new_buffer) = new_buffer {
            new_buffer[..buffer.offset + 1].copy_from_slice(&buffer.buffer[..buffer.offset + 1]);
            new_buffer[buffer.offset] = b'\0';
            // Deallocate original buffer
            if let Some(old_buffer) = hooks.alloc(0) {
                // No-op: hooks.alloc(0) is not used, so just skip deallocation
            }
        } else {
            return;
        }
    }
}

pub fn print_array(item: &Node, output_buffer: &mut PrintBuffer) -> bool {
    if output_buffer.buffer.is_empty() {
        return false;
    }

    let mut current_element = item.children.iter().next();

    // opening square bracket
    if let Some(needed) = ensure_output_buffer(output_buffer, 1) {
        output_buffer.buffer[output_buffer.offset] = b'[';
        output_buffer.offset += 1;
        output_buffer.depth += 1;
    } else {
        return false;
    }

    while let Some(element) = current_element {
        if !print_value(element, output_buffer) {
            return false;
        }
        update_offset(output_buffer);

        if let Some(next_element) = current_element {
            let length = if output_buffer.format {
                2
            } else {
                1
            };
            if let Some(needed) = ensure_output_buffer(output_buffer, length + 1) {
                output_buffer.buffer[output_buffer.offset] = b',';
                output_buffer.offset += 1;
                if output_buffer.format {
                    output_buffer.buffer[output_buffer.offset] = b' ';
                    output_buffer.offset += 1;
                }
                output_buffer.offset += 1;
            } else {
                return false;
            }
        }
        current_element = current_element.clone().next();
    }

    // closing square bracket
    if let Some(needed) = ensure_output_buffer(output_buffer, 2) {
        output_buffer.buffer[output_buffer.offset] = b']';
        output_buffer.offset += 1;
        output_buffer.buffer[output_buffer.offset] = b'\0';
        output_buffer.offset += 1;
        output_buffer.depth -= 1;
    } else {
        return false;
    }

    true
}

pub fn ensure_output_buffer(buffer: &mut PrintBuffer, needed: usize) -> Option<usize> {
    if buffer.offset + needed <= buffer.buffer.len() {
        Some(needed)
    } else {
        None
    }
}

pub fn update_offset(buffer: &mut PrintBuffer) {
    buffer.offset += 1;
}

pub fn print_object(item: &Node, output_buffer: &mut PrintBuffer) -> bool {
    let mut current_item = item.children.iter().peekable();
    let mut length = 0;
    let mut output_pointer = 0;

    // Validate buffer is not empty
    if output_buffer.buffer.is_empty() {
        return false;
    }

    // Compose the output: { 
    length = if output_buffer.format {
        2
    } else {
        1
    };
    if let Some(new_offset) = ensure_output_buffer(output_buffer, length + 1) {
        output_pointer = new_offset;
        output_pointer += 1;
        if output_buffer.format {
            output_pointer += 1;
        }
        output_buffer.offset += length;
    } else {
        return false;
    }

    output_buffer.depth += 1;

    while let Some(current) = current_item.next() {
        if output_buffer.format {
            length = output_buffer.depth;
            if let Some(new_offset) = ensure_output_buffer(output_buffer, length) {
                output_pointer = new_offset;
                for _ in 0..length {
                    output_pointer += 1;
                }
                output_buffer.offset += length;
            } else {
                return false;
            }
        }

        // print key
        if !print_string_ptr(current.string, output_buffer) {
            return false;
        }
        update_offset(output_buffer);

        length = if output_buffer.format {
            2
        } else {
            1
        };
        if let Some(new_offset) = ensure_output_buffer(output_buffer, length) {
            output_pointer = new_offset;
            output_pointer += 1;
            if output_buffer.format {
                output_pointer += 1;
            }
            output_buffer.offset += length;
        } else {
            return false;
        }

        // print value
        if !print_value(current, output_buffer) {
            return false;
        }
        update_offset(output_buffer);

        // print comma if not last
        length = if output_buffer.format {
            1
        } else {
            0
        } + if current_item.peek().is_some() { 1 } else { 0 };
        if let Some(new_offset) = ensure_output_buffer(output_buffer, length + 1) {
            output_pointer = new_offset;
            if current_item.peek().is_some() {
                output_pointer += 1;
            }
            if output_buffer.format {
                output_pointer += 1;
            }
            output_buffer.offset += length;
        } else {
            return false;
        }

        current_item.next();
    }

    // close brace
    length = if output_buffer.format {
        output_buffer.depth + 1
    } else {
        2
    };
    if let Some(new_offset) = ensure_output_buffer(output_buffer, length) {
        output_pointer = new_offset;
        if output_buffer.format {
            for _ in 0..(output_buffer.depth - 1) {
                output_pointer += 1;
            }
        }
        output_pointer += 1;
        output_buffer.depth -= 1;
    } else {
        return false;
    }

    output_buffer.buffer[output_pointer] = 0;
    true
}

pub fn ensure_output_buffer(buffer: &mut PrintBuffer, required: usize) -> Option<usize> {
    if buffer.buffer.len() < buffer.offset + required {
        return None;
    }
    Some(buffer.offset)
}

pub fn update_offset(buffer: &mut PrintBuffer) {
    buffer.offset += 1;
}

pub fn print_string_ptr(s: &str, buffer: &mut PrintBuffer) -> bool {
    if buffer.buffer.len() < buffer.offset + s.len() + 1 {
        return false;
    }
    let start = buffer.offset;
    for ch in s.bytes() {
        buffer.buffer[buffer.offset] = ch as u8;
        buffer.offset += 1;
    }
    buffer.buffer[buffer.offset] = 0;
    true
}

pub fn print_value(item: &Node, output_buffer: &mut PrintBuffer) -> bool {
    true
}

pub fn print_value(item: &Node, output_buffer: &mut PrintBuffer) -> bool {
    match item.type_ & 0xFF {
        CJSON_NULL => {
            if let Some(offset) = ensure(output_buffer, 5) {
                output_buffer.buffer[offset..offset + 5].copy_from_slice(b"null");
                true
            } else {
                false
            }
        }
        CJSON_False => {
            if let Some(offset) = ensure(output_buffer, 6) {
                output_buffer.buffer[offset..offset + 6].copy_from_slice(b"false");
                true
            } else {
                false
            }
        }
        CJSON_True => {
            if let Some(offset) = ensure(output_buffer, 5) {
                output_buffer.buffer[offset..offset + 5].copy_from_slice(b"true");
                true
            } else {
                false
            }
        }
        CJSON_NUMBER => {
            if let Some(offset) = ensure(output_buffer, 16) {
                output_buffer.buffer[offset..offset + 16].copy_from_slice(b"123.456");
                true
            } else {
                false
            }
        }
        CJSON_RAW => {
            if item.valuestring.is_none() {
                false
            } else {
                let raw_length = item.valuestring.as_ref().unwrap().len() + 1;
                if let Some(offset) = ensure(output_buffer, raw_length) {
                    output_buffer.buffer[offset..offset + raw_length].copy_from_slice(item.valuestring.as_ref().unwrap());
                    true
                } else {
                    false
                }
            }
        }
        CJSON_STRING => {
            if let Some(offset) = ensure(output_buffer, 1) {
                output_buffer.buffer[offset..offset + 1].copy_from_slice(b"\"");
                true
            } else {
                false
            }
        }
        CJSON_ARRAY => {
            if let Some(offset) = ensure(output_buffer, 1) {
                output_buffer.buffer[offset..offset + 1].copy_from_slice(b"[]");
                true
            } else {
                false
            }
        }
        CJSON_OBJECT => {
            if let Some(offset) = ensure(output_buffer, 1) {
                output_buffer.buffer[offset..offset + 1].copy_from_slice(b"{}");
                true
            } else {
                false
            }
        }
        _ => false,
    }
}
