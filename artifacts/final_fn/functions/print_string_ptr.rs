pub fn print_string_ptr(input: Option<&[u8]>, output_buffer: &mut PrintBuffer) -> bool {
    let input = match input {
        None => {
            let offset = match ensure(output_buffer, 2) {
                Some(off) => off,
                None => return false,
            };
            output_buffer.buffer[offset] = b'"';
            output_buffer.buffer[offset + 1] = b'"';
            output_buffer.offset = offset + 2;
            return true;
        }
        Some(s) => s,
    };

    let mut escape_chars = 0;
    for &b in input {
        match b {
            b'"' | b'\\' | 0x08 | 0x0C | b'\n' | b'\r' | b'\t' => escape_chars += 1,
            _ if b < 32 => escape_chars += 5,
            _ => {}
        }
    }

    let output_len = input.len() + escape_chars;
    let offset = match ensure(output_buffer, output_len + 2) {
        Some(off) => off,
        None => return false,
    };

    if escape_chars == 0 {
        output_buffer.buffer[offset] = b'"';
        output_buffer.buffer[offset + 1..offset + 1 + input.len()].copy_from_slice(input);
        output_buffer.buffer[offset + 1 + input.len()] = b'"';
        output_buffer.offset = offset + output_len + 2;
        return true;
    }

    let mut ptr = offset;
    output_buffer.buffer[ptr] = b'"';
    ptr += 1;

    for &b in input {
        if b > 31 && b != b'"' && b != b'\\' {
            output_buffer.buffer[ptr] = b;
            ptr += 1;
        } else {
            output_buffer.buffer[ptr] = b'\\';
            ptr += 1;
            match b {
                b'\\' => { output_buffer.buffer[ptr] = b'\\'; ptr += 1; }
                b'"' => { output_buffer.buffer[ptr] = b'"'; ptr += 1; }
                0x08 => { output_buffer.buffer[ptr] = b'b'; ptr += 1; }
                0x0C => { output_buffer.buffer[ptr] = b'f'; ptr += 1; }
                b'\n' => { output_buffer.buffer[ptr] = b'n'; ptr += 1; }
                b'\r' => { output_buffer.buffer[ptr] = b'r'; ptr += 1; }
                b'\t' => { output_buffer.buffer[ptr] = b't'; ptr += 1; }
                _ => {
                    output_buffer.buffer[ptr] = b'u';
                    ptr += 1;
                    let hex = format!("{:04x}", b);
                    output_buffer.buffer[ptr..ptr + 4].copy_from_slice(hex.as_bytes());
                    ptr += 4;
                }
            }
        }
    }
    output_buffer.buffer[ptr] = b'"';
    ptr += 1;
    output_buffer.offset = ptr;
    true
}
