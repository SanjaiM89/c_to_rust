use crate::contract::*;

pub fn ensure(p: &mut PrintBuffer, needed: usize) -> Option<usize> {
    if p.buffer.is_none() || p.buffer.as_ref().unwrap().is_empty() {
        return None;
    }

    if p.length > 0 && p.offset >= p.length {
        return None;
    }

    if needed > i32::MAX as usize {
        return None;
    }

    let new_needed = p.offset + 1 + needed;
    if new_needed <= p.length {
        return Some(p.offset);
    }

    if p.noalloc {
        return None;
    }

    let new_size = if new_needed > (i32::MAX / 2) as usize {
        if new_needed <= i32::MAX as usize {
            i32::MAX as usize
        } else {
            return None;
        }
    } else {
        new_needed * 2
    };

    if let Some(reallocate) = &p.hooks.reallocate {
        let new_buffer = reallocate(p.buffer.as_mut().unwrap(), new_size);
        if new_buffer.is_none() {
            p.buffer = None;
            p.length = 0;
            return None;
        }
        p.buffer = Some(new_buffer.unwrap());
        p.length = new_size;
        return Some(p.offset);
    } else if let Some(allocate) = &p.hooks.allocate {
        let new_buffer = allocate(new_size);
        if new_buffer.is_none() {
            p.buffer = None;
            p.length = 0;
            return None;
        }
        let old_buffer = p.buffer.take();
        if let Some(old) = old_buffer {
            let old_slice = &old[..p.offset + 1];
            new_buffer.unwrap().copy_from_slice(old_slice);
        }
        p.buffer = Some(new_buffer.unwrap());
        p.length = new_size;
        return Some(p.offset);
    }

    None
}

pub fn update_offset(p: &mut PrintBuffer) {
    if p.buffer.is_none() || p.buffer.as_ref().unwrap().is_empty() {
        return;
    }

    let buffer_ptr = &p.buffer.as_ref().unwrap()[p.offset..];
    p.offset += buffer_ptr.len();
}
