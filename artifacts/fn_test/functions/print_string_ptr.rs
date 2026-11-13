pub fn print_string_ptr(input: Option<&[u8]>, output_buffer: &mut PrintBuffer) -> bool {
    let input_pointer = input.as_ref().map_or(&[], |v| v);
    let mut escape_characters = 0;

    // Count escape characters needed
    for input_char in input_pointer.iter() {
        match *input_char {
            b'"' | b'\\' | b'\x08' | b'\x0C' | b'\n' | b'\r' | b'\t' => {
                escape_characters += 1;
            }
            b' ' => {
                // ASCII 32 is space, not escaped
            }
            _ if *input_char < 32 => {
                // UTF-16 escape sequence uXXXX
                escape_characters += 5;
            }
            _ => {}
        }
    }

    let output_length = input_pointer.len() + escape_characters;
    let required = output_length + 2; // for "..." (2 quotes)

    if required <= output_buffer.buffer.len() {
        let mut output = vec![0u8; required];
        output[0] = b'"';

        let mut output_ptr = &mut output[1..];
        let mut input_iter = input_pointer.iter();

        for input_char in input_iter {
            if *input_char > 31 && *input_char != b'"' && *input_char != b'\\' {
                output_ptr.push(*input_char);
            } else {
                output_ptr.push(b'\\');
                match *input_char {
                    b'\\' => output_ptr.push(b'\\'),
                    b'"' => output_ptr.push(b'"'),
                    b'\x08' => output_ptr.push(b'b'),
                    b'\x0C' => output_ptr.push(b'f'),
                    b'\n' => output_ptr.push(b'n'),
                    b'\r' => output_ptr.push(b'r'),
                    b'\t' => output_ptr.push(b't'),
                    _ => {
                        // UTF-16 escape sequence
                        let hex = format!("u{:04x}", *input_char as u32);
                        output_ptr.extend_from_slice(hex.as_bytes());
                    }
                }
            }
        }

        output_ptr.push(b'"');
        output_ptr.push(b'\0');

        // Update buffer
        output_buffer.offset = output.len();
        output_buffer.buffer = output;
        return true;
    }

    false
}
