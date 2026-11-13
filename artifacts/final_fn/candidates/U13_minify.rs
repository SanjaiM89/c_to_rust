// unit: U13_minify
pub fn minify(json: &mut Vec<u8>) {
    let mut into = Vec::with_capacity(json.len());
    let mut input = json.as_slice();

    while !input.is_empty() {
        match input[0] {
            b' ' | b'\t' | b'\r' | b'\n' => {
                input = &input[1..];
            }
            b'/' => {
                if input.len() > 1 && input[1] == b'/' {
                    skip_oneline_comment(&mut input);
                } else if input.len() > 1 && input[1] == b'*' {
                    skip_multiline_comment(&mut input);
                } else {
                    into.push(input[0]);
                    input = &input[1..];
                }
            }
            b'"' => {
                minify_string(&mut input, &mut into);
            }
            c => {
                into.push(c);
                input = &input[1..];
            }
        }
    }
    *json = into;
}

pub fn minify_string(input: &mut &[u8], output: &mut Vec<u8>) {
    if input.is_empty() {
        return;
    }
    output.push(input[0]);
    *input = &input[1..];

    while !input.is_empty() {
        let b = input[0];
        output.push(b);
        *input = &input[1..];
        if b == b'"' {
            return;
        } else if b == b'\\' && !input.is_empty() {
            output.push(input[0]);
            *input = &input[1..];
        }
    }
}

pub fn skip_multiline_comment(input: &mut &[u8]) {
    if input.starts_with(b"/*") {
        *input = &input[2..];
    }
    while input.len() >= 2 {
        if (*input)[0] == b'*' && (*input)[1] == b'/' {
            *input = &input[2..];
            return;
        }
        *input = &input[1..];
    }
    *input = &input[input.len()..];
}

pub fn skip_oneline_comment(input: &mut &[u8]) {
    if input.starts_with(b"//") {
        *input = &input[2..];
    }
    while !input.is_empty() {
        if (*input)[0] == b'\n' {
            *input = &input[1..];
            return;
        }
        *input = &input[1..];
    }
}
