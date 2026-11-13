// model: Qwen3-4B-Instruct-2507-Q4_K_M
// prompt_sha256: 869a457f6b85f97100f340912fb7607d027ce57b58994c0fa9c58b167b3a9498
// tokens in/out: 2045/301
// wall: 0.0s
// pub-normalised: True
pub fn ensure(p: &mut PrintBuffer, needed: usize) -> Option<usize> {
    if p.buffer.is_empty() || p.offset >= p.length {
        return None;
    }

    if needed > i32::MAX as usize {
        return None;
    }

    let new_needed = needed + p.offset + 1;
    if new_needed <= p.length {
        return Some(p.offset);
    }

    if p.noalloc {
        return None;
    }

    let new_size = if new_needed > (i32::MAX as usize / 2) {
        if new_needed <= i32::MAX as usize {
            i32::MAX as usize
        } else {
            return None;
        }
    } else {
        new_needed * 2
    };

    if new_size > i32::MAX as usize {
        return None;
    }

    let mut new_buf = if new_size > p.buffer.len() {
        vec![0u8; new_size]
    } else {
        p.buffer.clone()
    };

    if p.buffer.len() > 0 {
        new_buf[..p.buffer.len()].copy_from_slice(&p.buffer);
    }

    new_buf.resize(new_size, 0);

    p.buffer = new_buf;
    p.length = new_size;
    Some(p.offset)
}

pub fn update_offset(buffer: &mut PrintBuffer) {
    if buffer.buffer.is_empty() || buffer.offset >= buffer.length {
        return;
    }

    let buffer_ptr = &buffer.buffer[buffer.offset..];
    let len = buffer_ptr.len();
    if len > 0 {
        buffer.offset += len;
    }
}
