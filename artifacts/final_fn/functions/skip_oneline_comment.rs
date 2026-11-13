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
