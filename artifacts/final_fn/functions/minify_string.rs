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
