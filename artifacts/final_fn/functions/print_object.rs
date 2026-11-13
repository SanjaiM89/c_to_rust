pub fn print_object(item: &Node, output_buffer: &mut PrintBuffer) -> bool {
    let length = if output_buffer.format { 2 } else { 1 };
    let off = match ensure(output_buffer, length + 1) {
        Some(o) => o,
        None => return false,
    };
    output_buffer.buffer[off] = b'{';
    output_buffer.depth += 1;
    if output_buffer.format {
        output_buffer.buffer[off + 1] = b'\n';
    }
    output_buffer.offset += length;

    let total = item.children.len();
    for (idx, child) in item.children.iter().enumerate() {
        if output_buffer.format {
            let off = match ensure(output_buffer, output_buffer.depth) {
                Some(o) => o,
                None => return false,
            };
            output_buffer.buffer[off..off + output_buffer.depth].fill(b'\t');
            output_buffer.offset += output_buffer.depth;
        }

        let key = child.string.as_deref();
        if !print_string_ptr(key, output_buffer) {
            return false;
        }
        update_offset(output_buffer);

        let colon_len = if output_buffer.format { 2 } else { 1 };
        let off = match ensure(output_buffer, colon_len) {
            Some(o) => o,
            None => return false,
        };
        output_buffer.buffer[off] = b':';
        if output_buffer.format {
            output_buffer.buffer[off + 1] = b'\t';
        }
        output_buffer.offset += colon_len;

        if !print_value(child, output_buffer) {
            return false;
        }
        update_offset(output_buffer);

        let comma_len = (if output_buffer.format { 1 } else { 0 }) + (if idx + 1 < total { 1 } else { 0 });
        if comma_len > 0 {
            let off = match ensure(output_buffer, comma_len + 1) {
                Some(o) => o,
                None => return false,
            };
            let mut p = off;
            if idx + 1 < total {
                output_buffer.buffer[p] = b',';
                p += 1;
            }
            if output_buffer.format {
                output_buffer.buffer[p] = b'\n';
                p += 1;
            }
            output_buffer.offset += comma_len;
        }
    }

    let end_len = if output_buffer.format { output_buffer.depth + 1 } else { 2 };
    let off = match ensure(output_buffer, end_len) {
        Some(o) => o,
        None => return false,
    };
    let mut p = off;
    if output_buffer.format {
        let tabs = output_buffer.depth.saturating_sub(1);
        output_buffer.buffer[p..p + tabs].fill(b'\t');
        p += tabs;
    }
    output_buffer.buffer[p] = b'}';
    output_buffer.offset = p + 1;
    output_buffer.depth -= 1;
    true
}
