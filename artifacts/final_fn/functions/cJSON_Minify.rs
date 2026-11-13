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
