pub fn parse_number(item: &mut Node, input_buffer: &mut ParseBuffer<'_>) -> bool {
    let start = input_buffer.offset;
    if start >= input_buffer.length {
        return false;
    }

    let mut number_string_length = 0;
    let mut i = start;

    while i < input_buffer.length {
        let b = input_buffer.content[i];
        match b {
            b'0'..=b'9' | b'+' | b'-' | b'e' | b'E' | b'.' => {
                number_string_length += 1;
            }
            _ => {
                break;
            }
        }
        i += 1;
    }

    if number_string_length == 0 {
        return false;
    }

    let slice = &input_buffer.content[start..start + number_string_length];
    let s = match std::str::from_utf8(slice) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let mut parsed: Option<(f64, usize)> = None;
    for len in (1..=s.len()).rev() {
        if let Ok(num) = s[..len].parse::<f64>() {
            parsed = Some((num, len));
            break;
        }
    }

    let (number, len) = match parsed {
        Some(p) => p,
        None => return false,
    };

    item.valuedouble = number;
    if number >= i32::MAX as f64 {
        item.valueint = i32::MAX;
    } else if number <= i32::MIN as f64 {
        item.valueint = i32::MIN;
    } else {
        item.valueint = number as i32;
    }

    item.type_ = CJSON_NUMBER;
    input_buffer.offset += len;
    true
}
