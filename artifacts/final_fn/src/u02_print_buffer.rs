//@unit: U02_print_buffer
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

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

pub fn update_offset(buffer: &mut PrintBuffer) {
    if buffer.offset >= buffer.buffer.len() {
        return;
    }
    let slice = &buffer.buffer[buffer.offset..];
    let len = slice.iter().position(|&b| b == 0).unwrap_or(0);
    buffer.offset += len;
}

