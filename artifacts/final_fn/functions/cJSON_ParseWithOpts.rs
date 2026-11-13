pub fn parse_with_opts(
    value: &[u8],
    return_parse_end: Option<&mut usize>,
    require_null_terminated: bool,
) -> Option<Node> {
    parse_with_length_opts(value, value.len(), return_parse_end, require_null_terminated)
}
