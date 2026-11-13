use crate::contract::*;

pub fn set_number_helper(object: &mut Node, number: f64) -> f64 {
    if number >= i32::MAX as f64 {
        object.valueint = i32::MAX;
    } else if number <= (i32::MIN as f64) {
        object.valueint = i32::MIN;
    } else {
        object.valueint = (number as i32);
    }

    object.valuedouble = number;
    number
}

pub fn compare_double(a: f64, b: f64) -> bool {
    let max_val = (a.abs().max(b.abs()));
    (a - b).abs() <= max_val * f64::EPSILON
}

pub fn get_decimal_point() -> u8 {
    // In Rust, we don't have locale support via localeconv directly,
    // and the C function uses localeconv which is not available in standard Rust.
    // Since the C code falls back to '.' when locales are disabled,
    // and there's no equivalent locale configuration in Rust without external crates,
    // we default to the fallback behavior.
    b'.'
}

pub fn parse_number(item: &mut Node, input_buffer: &mut ParseBuffer<'_>) -> bool {
    let decimal_point = b'.';
    let mut number_string_length = 0;
    let mut has_decimal_point = false;
    let mut i = 0;

    // Count the number of characters in the number string
    while i < input_buffer.content.len() {
        let b = input_buffer.content[i];
        match b {
            b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' |
            b'+' | b'-' | b'e' | b'E' => {
                number_string_length += 1;
            }
            b'.' => {
                number_string_length += 1;
                has_decimal_point = true;
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

    // Allocate temporary buffer for the number string
    let mut number_c_string = vec![0u8; number_string_length + 1];
    // Copy the input into the temporary buffer
    let input_slice = &input_buffer.content[..i];
    number_c_string[..number_string_length].copy_from_slice(input_slice);
    number_c_string[number_string_length] = b'\0';

    // Replace '.' with the locale decimal point if present
    if has_decimal_point {
        for j in 0..number_string_length {
            if number_c_string[j] == b'.' {
                number_c_string[j] = decimal_point;
            }
        }
    }

    // Parse the number using strtod equivalent
    let after_end = std::str::from_utf8(&number_c_string[..number_string_length])
        .map(|s| {
            let number = s.parse::<f64>().map_err(|_| ()).unwrap_or(0.0);
            let after_end = s.as_bytes().iter().position(|&b| b == 0).unwrap_or(s.len());
            after_end
        })
        .map_err(|_| ())
        .unwrap_or(0);

    if after_end == 0 {
        return false;
    }

    // Update the item with parsed number
    let number = std::str::from_utf8(&number_c_string[..after_end])
        .map(|s| s.parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0);
    item.valuedouble = number;

    // Convert to integer with saturation
    if number >= i32::MAX as f64 {
        item.valueint = i32::MAX;
    } else if number <= i32::MIN as f64 {
        item.valueint = i32::MIN;
    } else {
        item.valueint = number as i32;
    }

    item.type_ = CJSON_NUMBER;

    // Update the buffer offset
    input_buffer.offset += after_end;

    true
}
