pub fn parse_with_length(value: &[u8], buffer_length: usize) -> Option<Node> {
    parse_with_length_opts(value, buffer_length, None, false)
}
