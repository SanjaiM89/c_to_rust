pub fn update_offset(buffer: &mut PrintBuffer) {
    if buffer.buffer.is_empty() {
        return;
    }
    let buffer_pointer = &buffer.buffer[buffer.offset..];
    let len = buffer_pointer.len();
    if len == 0 {
        return;
    }
    buffer.offset += len;
}
