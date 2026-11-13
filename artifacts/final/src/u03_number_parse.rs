//@unit: U03_number_parse
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

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

pub use set_number_helper as cJSON_SetNumberHelper;
