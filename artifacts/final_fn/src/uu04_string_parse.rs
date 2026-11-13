use crate::contract::*;

pub fn parse_hex4(input: &[u8]) -> u32 {
    let mut h = 0u32;
    for i in 0..4 {
        let b = input[i];
        if (b >= b'0' && b <= b'9') {
            h += (b - b'0') as u32;
        } else if (b >= b'A' && b <= b'F') {
            h += (10 + b - b'A') as u32;
        } else if (b >= b'a' && b <= b'f') {
            h += (10 + b - b'a') as u32;
        } else {
            return 0;
        }

        if i < 3 {
            h <<= 4;
        }
    }
    h
}

pub fn parse_string(item: &mut Node, input_buffer: &mut ParseBuffer<'_>) -> bool {
    let input_start = input_buffer.offset + 1;
    let input_slice = &input_buffer.content[input_buffer.offset + 1..];
    if input_slice.is_empty() || input_buffer.content[input_buffer.offset] != b'"' {
        return false;
    }

    let mut output = vec![0u8; 0];
    let mut allocation_length = 0;
    let mut skipped_bytes = 0;

    let mut end = input_slice.as_ptr() as *const u8;
    while end as usize < input_buffer.length && *end != b'"' {
        if *end == b'\\' {
            if end as usize + 1 >= input_buffer.length {
                return false;
            }
            skipped_bytes += 1;
            end = end.offset(1);
        }
        end = end.offset(1);
    }

    if end as usize >= input_buffer.length || *end != b'"' {
        return false;
    }

    let allocation_length = (end as usize - input_start) - skipped_bytes;
    let output_capacity = allocation_length + 1; // +1 for null terminator
    let mut output = vec![0u8; output_capacity];

    let mut input_ptr = input_slice.as_ptr() as *const u8;
    let mut output_ptr = &mut output[0] as *mut u8;

    while input_ptr < end {
        if *input_ptr != b'\\' {
            *output_ptr = *input_ptr;
            output_ptr = output_ptr.offset(1);
            input_ptr = input_ptr.offset(1);
        } else {
            let sequence_length = 2;
            if end as usize - (input_ptr as usize) < 1 {
                return false;
            }

            let ch = *input_ptr.offset(1);
            match ch {
                b'b' => *output_ptr = b'\x08',
                b'f' => *output_ptr = b'\x0C',
                b'n' => *output_ptr = b'\n',
                b'r' => *output_ptr = b'\r',
                b't' => *output_ptr = b'\t',
                b'"' | b'\\' | b'/' => {
                    *output_ptr = ch;
                }
                b'u' => {
                    let len = utf16_literal_to_utf8(&input_slice[input_ptr.offset(1) as usize..], end, &mut output);
                    if len == 0 {
                        return false;
                    }
                    sequence_length = len;
                }
                _ => return false,
            }

            input_ptr = input_ptr.offset(sequence_length);
            output_ptr = output_ptr.offset(1);
        }
    }

    *output_ptr = b'\0';

    item.type_ = CJSON_STRING;
    item.valuestring = Some(output);

    input_buffer.offset = (end as usize - input_buffer.offset) + 1;

    true
}

pub fn utf16_literal_to_utf8(input_pointer: &[u8], input_end: &[u8], output_pointer: &mut Vec<u8>) -> Option<Vec<u8>> {
    if input_end.len() < 6 {
        return None;
    }

    let first_sequence = input_pointer;
    let first_code = parse_hex4(&first_sequence[2..6]);

    // Check for invalid initial UTF-16 code point
    if (first_code >= 0xDC00 && first_code <= 0xDFFF) {
        return None;
    }

    let mut codepoint = 0u32;
    let mut sequence_length = 0;
    let mut utf8_length = 0;
    let mut first_byte_mark = 0u8;

    if (first_code >= 0xD800 && first_code <= 0xDBFF) {
        // UTF-16 surrogate pair
        let second_sequence = &first_sequence[6..];
        if input_end.len() < 6 {
            return None;
        }

        if second_sequence[0] != b'\\' || second_sequence[1] != b'u' {
            return None;
        }

        let second_code = parse_hex4(&second_sequence[2..6]);
        if second_code < 0xDC00 || second_code > 0xDFFF {
            return None;
        }

        codepoint = 0x10000 + (((first_code & 0x3FF) << 10) | (second_code & 0x3FF));
        sequence_length = 12;
    } else {
        codepoint = first_code;
        sequence_length = 6;
    }

    // Encode as UTF-8
    if codepoint < 0x80 {
        utf8_length = 1;
    } else if codepoint < 0x800 {
        utf8_length = 2;
        first_byte_mark = 0xC0;
    } else if codepoint < 0x10000 {
        utf8_length = 3;
        first_byte_mark = 0xE0;
    } else if codepoint <= 0x10FFFF {
        utf8_length = 4;
        first_byte_mark = 0xF0;
    } else {
        return None;
    }

    // Allocate output buffer
    let mut output = vec![0u8; utf8_length];
    let mut pos = utf8_length - 1;

    while pos > 0 {
        output[pos] = (codepoint & 0xBF) as u8;
        codepoint >>= 6;
        pos -= 1;
    }

    if utf8_length > 1 {
        output[0] = (codepoint | first_byte_mark) as u8;
    } else {
        output[0] = codepoint as u8 & 0x7F;
    }

    // Update output_pointer to point to the new position
    *output_pointer = output;

    Some(vec![0u8; sequence_length])
}
