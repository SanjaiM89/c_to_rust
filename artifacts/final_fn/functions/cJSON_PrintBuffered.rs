pub fn print_buffered(item: &Node, prebuffer: usize, fmt: bool) -> Option<Vec<u8>> {
    let mut print_buffer = PrintBuffer {
        buffer: vec![0u8; prebuffer.max(256)],
        length: prebuffer.max(256),
        offset: 0,
        depth: 0,
        noalloc: false,
        format: fmt,
        hooks: Hooks::default(),
    };

    if !print_value(item, &mut print_buffer) {
        return None;
    }
    update_offset(&mut print_buffer);
    let mut out = print_buffer.buffer;
    out.truncate(print_buffer.offset);
    Some(out)
}
