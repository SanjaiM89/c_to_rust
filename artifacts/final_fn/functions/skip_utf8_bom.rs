pub fn skip_utf8_bom(buffer: &mut ParseBuffer<'_>) {
    if buffer.offset + 3 <= buffer.length && &buffer.content[buffer.offset..buffer.offset + 3] == b"\xEF\xBB\xBF" {
        buffer.offset += 3;
    }
}
