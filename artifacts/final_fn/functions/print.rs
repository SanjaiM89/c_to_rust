pub fn print_internal(item: &Node, format: bool, hooks: &Hooks) -> Option<Vec<u8>> {
    let mut buffer = PrintBuffer {
        buffer: vec![0u8; 256],
        length: 256,
        offset: 0,
        depth: 0,
        noalloc: false,
        format,
        hooks: *hooks,
    };
    if !print_value(item, &mut buffer) {
        return None;
    }
    update_offset(&mut buffer);
    let mut out = buffer.buffer;
    out.truncate(buffer.offset);
    Some(out)
}
