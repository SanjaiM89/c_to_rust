pub fn ensure(p: &mut PrintBuffer, needed: usize) -> Option<usize> {
    if p.buffer.is_empty() && p.offset > 0 {
        return None;
    }

    if p.offset >= p.length {
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

    let new_size = if new_needed > (i32::MAX as usize) / 2 {
        if new_needed <= i32::MAX as usize {
            i32::MAX as usize
        } else {
            return None;
        }
    } else {
        new_needed * 2
    };

    if new_size > p.length {
        let mut new_buffer = p.buffer.clone();
        new_buffer.resize(new_size, 0);
        p.buffer = new_buffer;
        p.length = new_size;
    }

    Some(p.offset)
}
