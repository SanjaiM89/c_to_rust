pub fn parse(value: &[u8]) -> Option<Node> {
    parse_with_opts(value, None, false)
}
