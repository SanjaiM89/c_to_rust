use crate::contract::*;

pub fn ensure(p: &mut PrintBuffer, needed: usize) -> Option<usize> {
    if p.buffer.is_null() {
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
        let new_buffer = reallocate(p.buffer.as_mut_ptr() as *mut _, new_size);
        if new_buffer.is_null() {
            p.length = 0;
            p.buffer = std::ptr::null_mut();
            return None;
        }
        p.buffer = new_buffer;
        p.length = new_size;
        return Some(p.offset);
    } else if let Some(allocate) = &p.hooks.allocate {
        let new_buffer = allocate(new_size);
        if new_buffer.is_null() {
            p.length = 0;
            p.buffer = std::ptr::null_mut();
            return None;
        }

        let old_buffer = std::mem::replace(&mut p.buffer, std::ptr::null_mut());
        if !old_buffer.is_null() {
            let _ = old_buffer;
            // Copy the existing data
            let len = p.offset + 1;
            let src = &old_buffer[..len];
            new_buffer.copy_from_slice(src);
        }
        p.buffer = new_buffer;
        p.length = new_size;
        return Some(p.offset);
    }

    None
}

pub fn update_offset(buffer: &mut PrintBuffer) {
    if buffer.buffer.is_null() {
        return;
    }

    let buffer_pointer = &buffer.buffer[buffer.offset..];
    buffer.offset += buffer_pointer.len();
}
