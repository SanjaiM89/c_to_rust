pub fn buffer_skip_whitespace(buffer: &mut ParseBuffer<'_>) {
    while buffer.offset < buffer.length && buffer.content[buffer.offset] <= 32 {
        buffer.offset += 1;
    }
    if buffer.offset >= buffer.length {
        buffer.offset = buffer.length;
    }
}
