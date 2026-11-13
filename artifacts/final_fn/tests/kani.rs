//! Kani proof harnesses for the highest-risk functions.
//!
//! The brief asks for bounded proofs on "bounds, integer overflow and
//! buffer-growth invariants". These harnesses target exactly those:
//!
//!   * `ensure`        - the printbuffer growth invariant (never writes past
//!                        the end, never overflows the size computation)
//!   * `parse_number`  - the numeric scanner (no out-of-bounds read, the
//!                        integer truncation matches C)
//!   * `parse_hex4`    - a fixed 4-byte read, must never read out of bounds
//!   * `print_number`  - the number formatter must stay inside the buffer
//!   * `update_offset` - the cursor advance must stay inside the buffer
//!
//! Run with:  cargo kani --tests   (or `cargo kani` inside the crate)

#![cfg(kani)]

use cjson_rs::*;

/// The printbuffer growth invariant: after `ensure`, there is room for
/// `needed` bytes at `offset`, and the offset never moves backwards.
#[kani::proof]
fn ensure_keeps_the_buffer_consistent() {
    let needed: usize = kani::any();
    let offset: usize = kani::any();
    kani::assume(needed <= 4096);
    kani::assume(offset <= 4096);

    let mut p = PrintBuffer {
        buffer: vec![0u8; 64],
        length: 64,
        offset,
        depth: 0,
        noalloc: false,
        format: false,
        hooks: Hooks::default(),
    };
    let before = p.offset;
    let r = cjson_rs::u02_print_buffer::ensure(&mut p, needed);
    match r {
        Some(idx) => {
            // the returned index is the write cursor
            assert!(idx == before, "ensure must return the current offset");
            // and the buffer is now large enough for `needed` bytes
            assert!(
                p.buffer.len() >= p.offset + needed,
                "ensure must guarantee room for `needed` bytes"
            );
            // the offset never moves backwards
            assert!(p.offset >= before, "ensure must not move the offset back");
        }
        None => {
            // the only legitimate failure is `noalloc`
            assert!(p.noalloc, "ensure may only fail when noalloc is set");
        }
    }
}

/// `parse_hex4` reads exactly four bytes; it must never read past the end of
/// the input and must return 0 for invalid digits.
#[kani::proof]
fn parse_hex4_never_reads_out_of_bounds() {
    let input: [u8; 4] = kani::any();
    let v = cjson_rs::u04_string_parse::parse_hex4(&input);
    // any 4 hex digits must decode to the expected value
    let all_hex = input.iter().all(|c| c.is_ascii_hexdigit());
    if all_hex {
        let mut expect: u32 = 0;
        for c in input.iter() {
            expect = expect * 16 + (*c as char).to_digit(16).unwrap();
        }
        assert_eq!(v, expect, "hex4 must decode correctly");
    } else {
        assert_eq!(v, 0, "invalid hex digits must decode to 0");
    }
}

/// The numeric scanner must never read past `content.len()`.
#[kani::proof]
fn parse_number_stays_in_bounds() {
    let s: [u8; 8] = kani::any();
    let mut item = Node::new(CJSON_NUMBER);
    let mut buf = ParseBuffer::new(&s);
    let r = cjson_rs::u03_number_parse::parse_number(&mut item, &mut buf);
    // whether it succeeds or not, the cursor must stay inside the buffer
    assert!(buf.offset <= buf.length, "the cursor must stay inside the buffer");
    if r {
        assert_eq!(item.type_ & 0xFF, CJSON_NUMBER, "a parsed number is a number");
    }
}

/// The number formatter must not write outside the printbuffer.
#[kani::proof]
fn print_number_stays_in_bounds() {
    let value: f64 = kani::any();
    let mut item = Node::new(CJSON_NUMBER);
    item.valuedouble = value;
    item.valueint = value as i32;
    let mut buf = PrintBuffer {
        buffer: vec![0u8; 64],
        length: 64,
        offset: 0,
        depth: 0,
        noalloc: true, // forbid growth: any overflow must be detected, not absorbed
        format: false,
        hooks: Hooks::default(),
    };
    let r = cjson_rs::u07_value_print_p1::print_number(&item, &mut buf);
    assert!(buf.offset <= buf.buffer.len(), "the write cursor must stay inside");
    if r {
        assert!(buf.offset <= 64, "the formatted number must fit the buffer");
    }
}

/// `update_offset` advances the cursor by the length of the string at the
/// cursor; it must never advance past the end of the buffer.
#[kani::proof]
fn update_offset_stays_in_bounds() {
    let content: [u8; 8] = kani::any();
    let offset: usize = kani::any();
    kani::assume(offset <= content.len());
    let mut buf = PrintBuffer {
        buffer: content.to_vec(),
        length: content.len(),
        offset,
        depth: 0,
        noalloc: true,
        format: false,
        hooks: Hooks::default(),
    };
    cjson_rs::u02_print_buffer::update_offset(&mut buf);
    assert!(buf.offset <= buf.buffer.len(), "the cursor must stay inside");
}

/// The integer truncation of a number node must match the C behaviour
/// (truncation towards zero) and must not overflow.
#[kani::proof]
fn number_truncation_matches_c() {
    let value: f64 = kani::any();
    let mut item = Node::new(CJSON_NUMBER);
    cjson_rs::u03_number_parse::cJSON_SetNumberHelper(&mut item, value);
    // valueint is the C truncation of valuedouble towards zero
    let expected = value as i32;
    assert_eq!(item.valueint, expected, "valueint must be the C truncation");
    assert_eq!(item.valuedouble, value, "valuedouble must be preserved");
}
