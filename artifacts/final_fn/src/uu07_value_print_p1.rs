use crate::contract::*;

pub fn print_number(item: &Node, output_buffer: &mut PrintBuffer) -> bool {
    let d = item.valuedouble;
    let mut number_buffer = [0u8; 26];
    let mut test: f64 = 0.0;
    let mut length = 0;
    let mut i = 0;

    // Check for NaN and Infinity
    if d.is_nan() || d.is_infinite() {
        length = 4; // "null" has 4 bytes
    } else if d == item.valueint as f64 {
        let s = format!("{}", item.valueint);
        length = s.len();
        if length >= number_buffer.len() {
            return false;
        }
        let mut output_pointer = &mut output_buffer.buffer[output_buffer.offset..];
        output_pointer[..length].copy_from_slice(s.as_bytes());
        output_buffer.offset += length;
        return true;
    } else {
        // Try 15 decimal places of precision
        let formatted = format!("{:.15}", d);
        let parsed: Result<f64, _> = formatted.parse();
        if parsed.is_ok() {
            let parsed_val = parsed.unwrap();
            if parsed_val == d {
                length = formatted.len();
                if length >= number_buffer.len() {
                    return false;
                }
                let mut output_pointer = &mut output_buffer.buffer[output_buffer.offset..];
                for i in 0..length {
                    if number_buffer[i] == b'.' {
                        output_pointer[i] = b'.';
                    } else {
                        output_pointer[i] = number_buffer[i];
                    }
                }
                output_pointer[length] = b'\0';
                output_buffer.offset += length;
                return true;
            }
        }

        // Fall back to 17 decimal places
        let formatted = format!("{:.17}", d);
        let parsed: Result<f64, _> = formatted.parse();
        if parsed.is_ok() {
            let parsed_val = parsed.unwrap();
            if parsed_val == d {
                length = formatted.len();
                if length >= number_buffer.len() {
                    return false;
                }
                let mut output_pointer = &mut output_buffer.buffer[output_buffer.offset..];
                for i in 0..length {
                    if number_buffer[i] == b'.' {
                        output_pointer[i] = b'.';
                    } else {
                        output_pointer[i] = number_buffer[i];
                    }
                }
                output_pointer[length] = b'\0';
                output_buffer.offset += length;
                return true;
            }
        }
    }

    false
}
