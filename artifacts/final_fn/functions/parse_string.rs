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
