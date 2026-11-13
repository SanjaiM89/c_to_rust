pub fn print_array(item: &Node, output_buffer: &mut PrintBuffer) -> bool {
    let offset = match ensure(output_buffer, 1) {
        Some(off) => off,
        None => return false,
    };
    output_buffer.buffer[offset] = b'[';
    output_buffer.offset += 1;
    output_buffer.depth += 1;

    let len = item.children.len();
    for (i, child) in item.children.iter().enumerate() {
        if !print_value(child, output_buffer) {
            return false;
        }
        update_offset(output_buffer);
        if i + 1 < len {
            let length = if output_buffer.format { 2 } else { 1 };
            let off = match ensure(output_buffer, length + 1) {
                Some(o) => o,
                None => return false,
            };
            output_buffer.buffer[off] = b',';
            if output_buffer.format {
                output_buffer.buffer[off + 1] = b' ';
            }
            output_buffer.offset += length;
        }
    }

    let off = match ensure(output_buffer, 2) {
        Some(o) => o,
        None => return false,
    };
    output_buffer.buffer[off] = b']';
    output_buffer.offset += 1;
    output_buffer.depth -= 1;
    true
}
