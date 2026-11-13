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
