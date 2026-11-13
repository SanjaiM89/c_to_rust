// unit: U07_value_print_p2
pub fn print(item: &Node) -> Option<Vec<u8>> {
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        print_internal(item, true, &gh)
    })
}

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

pub fn print_value(item: &Node, output_buffer: &mut PrintBuffer) -> bool {
    match item.type_ & 0xFF {
        CJSON_NULL => {
            let offset = match ensure(output_buffer, 5) {
                Some(off) => off,
                None => return false,
            };
            output_buffer.buffer[offset..offset + 4].copy_from_slice(b"null");
            output_buffer.offset = offset + 4;
            true
        }
        CJSON_FALSE => {
            let offset = match ensure(output_buffer, 6) {
                Some(off) => off,
                None => return false,
            };
            output_buffer.buffer[offset..offset + 5].copy_from_slice(b"false");
            output_buffer.offset = offset + 5;
            true
        }
        CJSON_TRUE => {
            let offset = match ensure(output_buffer, 5) {
                Some(off) => off,
                None => return false,
            };
            output_buffer.buffer[offset..offset + 4].copy_from_slice(b"true");
            output_buffer.offset = offset + 4;
            true
        }
        CJSON_NUMBER => print_number(item, output_buffer),
        CJSON_RAW => {
            let val = match item.valuestring.as_ref() {
                Some(v) => v,
                None => return false,
            };
            let offset = match ensure(output_buffer, val.len() + 1) {
                Some(off) => off,
                None => return false,
            };
            output_buffer.buffer[offset..offset + val.len()].copy_from_slice(val);
            output_buffer.offset = offset + val.len();
            true
        }
        CJSON_STRING => print_string(item, output_buffer),
        CJSON_ARRAY => print_array(item, output_buffer),
        CJSON_OBJECT => print_object(item, output_buffer),
        _ => false,
    }
}
