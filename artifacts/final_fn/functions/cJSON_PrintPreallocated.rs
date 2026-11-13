pub fn print_preallocated(item: &Node, buffer: &mut [u8], length: i32, format: bool) -> bool {
    if length < 0 || (length as usize) > buffer.len() {
        return false;
    }

    let mut print_buffer = PrintBuffer {
        buffer: buffer[..length as usize].to_vec(),
        length: length as usize,
        offset: 0,
        depth: 0,
        noalloc: true,
        format,
        hooks: Hooks::default(),
    };

    if !print_value(item, &mut print_buffer) {
        return false;
    }

    update_offset(&mut print_buffer);
    let n = print_buffer.offset.min(buffer.len());
    buffer[..n].copy_from_slice(&print_buffer.buffer[..n]);
    if n < buffer.len() {
        buffer[n] = 0;
    }
    true
}
