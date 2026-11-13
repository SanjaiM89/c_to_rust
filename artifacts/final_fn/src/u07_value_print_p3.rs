//@unit: U07_value_print_p3
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

pub fn print_buffered(item: &Node, prebuffer: usize, fmt: bool) -> Option<Vec<u8>> {
    let mut print_buffer = PrintBuffer {
        buffer: vec![0u8; prebuffer.max(256)],
        length: prebuffer.max(256),
        offset: 0,
        depth: 0,
        noalloc: false,
        format: fmt,
        hooks: Hooks::default(),
    };

    if !print_value(item, &mut print_buffer) {
        return None;
    }
    update_offset(&mut print_buffer);
    let mut out = print_buffer.buffer;
    out.truncate(print_buffer.offset);
    Some(out)
}

pub fn print_preallocated(item: &Node, buffer: &mut [u8], length: i32, format: bool) -> bool {
    if length < 0 || (length as usize) > buffer.len() {
        return false;
    }

    let mut print_buffer = PrintBuffer {
        buffer: buffer[..length as usize].to_vec(),
        length: length as usize,
        offset: 0,
        depth: 0,
        noalloc: true,
        format,
        hooks: Hooks::default(),
    };

    if !print_value(item, &mut print_buffer) {
        return false;
    }

    update_offset(&mut print_buffer);
    let n = print_buffer.offset.min(buffer.len());
    buffer[..n].copy_from_slice(&print_buffer.buffer[..n]);
    if n < buffer.len() {
        buffer[n] = 0;
    }
    true
}

pub fn print_unformatted(item: &Node) -> Option<Vec<u8>> {
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        print_internal(item, false, &gh)
    })
}

pub use print_buffered as cJSON_PrintBuffered;
pub use print_preallocated as cJSON_PrintPreallocated;
pub use print_unformatted as cJSON_PrintUnformatted;
