pub fn print_string(item: &Node, p: &mut PrintBuffer) -> bool {
    print_string_ptr(item.valuestring.as_deref(), p)
}
