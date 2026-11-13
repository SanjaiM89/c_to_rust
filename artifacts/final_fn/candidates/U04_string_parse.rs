// unit: U04_string_parse
pub fn parse_hex4(input: &[u8]) -> u32 {
    if input.len() < 4 {
        return 0;
    }
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
    let start = input_buffer.offset;
    if start >= input_buffer.length {
        return false;
    }
    input_buffer.offset += 1;
    if input_buffer.content[start] != b'"' {
        return false;
    }

    let str_start = start + 1;
    let mut end = str_start;
    let mut skipped = 0;
    while end < input_buffer.length && input_buffer.content[end] != b'"' {
        if input_buffer.content[end] == b'\\' {
            if end + 1 >= input_buffer.length {
                return false;
            }
            skipped += 1;
            end += 1;
        }
        end += 1;
    }
    if end >= input_buffer.length || input_buffer.content[end] != b'"' {
        return false;
    }

    let mut output = Vec::with_capacity(end.saturating_sub(str_start).saturating_sub(skipped));
    let mut i = str_start;
    while i < end {
        if input_buffer.content[i] != b'\\' {
            output.push(input_buffer.content[i]);
            i += 1;
        } else {
            if i + 1 >= end {
                return false;
            }
            let esc = input_buffer.content[i + 1];
            match esc {
                b'b' => { output.push(0x08); i += 2; }
                b'f' => { output.push(0x0C); i += 2; }
                b'n' => { output.push(b'\n'); i += 2; }
                b'r' => { output.push(b'\r'); i += 2; }
                b't' => { output.push(b'\t'); i += 2; }
                b'"' | b'\\' | b'/' => { output.push(esc); i += 2; }
                b'u' => {
                    let seq = utf16_literal_to_utf8(
                        &input_buffer.content[i..],
                        &input_buffer.content[end..],
                        &mut output,
                    );
                    match seq {
                        Some(v) => i += v.len(),
                        None => return false,
                    }
                }
                _ => return false,
            }
        }
    }

    item.type_ = CJSON_STRING;
    item.valuestring = Some(output);
    input_buffer.offset = end + 1;
    true
}

pub fn utf16_literal_to_utf8(
    input_pointer: &[u8],
    _input_end: &[u8],
    output_pointer: &mut Vec<u8>,
) -> Option<Vec<u8>> {
    if input_pointer.len() < 6 {
        return None;
    }
    let first_code = parse_hex4(&input_pointer[2..6]);
    if (0xDC00..=0xDFFF).contains(&first_code) {
        return None;
    }
    let (codepoint, seq_len) = if (0xD800..=0xDBFF).contains(&first_code) {
        if input_pointer.len() < 12 {
            return None;
        }
        if input_pointer[6] != b'\\' || input_pointer[7] != b'u' {
            return None;
        }
        let second_code = parse_hex4(&input_pointer[8..12]);
        if !(0xDC00..=0xDFFF).contains(&second_code) {
            return None;
        }
        let cp = 0x10000 + (((first_code & 0x3FF) << 10) | (second_code & 0x3FF));
        (cp, 12)
    } else {
        (first_code, 6)
    };

    if codepoint < 0x80 {
        output_pointer.push(codepoint as u8);
    } else if codepoint < 0x800 {
        output_pointer.push((0xC0 | (codepoint >> 6)) as u8);
        output_pointer.push((0x80 | (codepoint & 0x3F)) as u8);
    } else if codepoint < 0x10000 {
        output_pointer.push((0xE0 | (codepoint >> 12)) as u8);
        output_pointer.push((0x80 | ((codepoint >> 6) & 0x3F)) as u8);
        output_pointer.push((0x80 | (codepoint & 0x3F)) as u8);
    } else if codepoint <= 0x10FFFF {
        output_pointer.push((0xF0 | (codepoint >> 18)) as u8);
        output_pointer.push((0x80 | ((codepoint >> 12) & 0x3F)) as u8);
        output_pointer.push((0x80 | ((codepoint >> 6) & 0x3F)) as u8);
        output_pointer.push((0x80 | (codepoint & 0x3F)) as u8);
    } else {
        return None;
    }

    Some(vec![0u8; seq_len])
}
