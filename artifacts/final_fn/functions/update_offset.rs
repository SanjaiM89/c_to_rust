pub fn update_offset(buffer: &mut PrintBuffer) {
    if buffer.offset >= buffer.buffer.len() {
        return;
    }
    let slice = &buffer.buffer[buffer.offset..];
    let len = slice.iter().position(|&b| b == 0).unwrap_or(0);
    buffer.offset += len;
}
