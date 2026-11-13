use crate::contract::*;
use crate::stubs::*;

pub fn utf16_literal_to_utf8(
    input_pointer: &[u8],
    _input_end: &[u8],
    output_pointer: &mut Vec<u8>,
) -> Option<Vec<u8>> {
    if input_pointer.len() < 6 {
        return None;
    }
    let first_code = parse_hex4(&input_pointer[2..6]);
    if (0xDC00..=0xDFFF).contains(&first_code) {
        return None;
    }
    let (codepoint, seq_len) = if (0xD800..=0xDBFF).contains(&first_code) {
        if input_pointer.len() < 12 {
            return None;
        }
        if input_pointer[6] != b'\\' || input_pointer[7] != b'u' {
            return None;
        }
        let second_code = parse_hex4(&input_pointer[8..12]);
        if !(0xDC00..=0xDFFF).contains(&second_code) {
            return None;
        }
        let cp = 0x10000 + (((first_code & 0x3FF) << 10) | (second_code & 0x3FF));
        (cp, 12)
    } else {
        (first_code, 6)
    };

    if codepoint < 0x80 {
        output_pointer.push(codepoint as u8);
    } else if codepoint < 0x800 {
        output_pointer.push((0xC0 | (codepoint >> 6)) as u8);
        output_pointer.push((0x80 | (codepoint & 0x3F)) as u8);
    } else if codepoint < 0x10000 {
        output_pointer.push((0xE0 | (codepoint >> 12)) as u8);
        output_pointer.push((0x80 | ((codepoint >> 6) & 0x3F)) as u8);
        output_pointer.push((0x80 | (codepoint & 0x3F)) as u8);
    } else if codepoint <= 0x10FFFF {
        output_pointer.push((0xF0 | (codepoint >> 18)) as u8);
        output_pointer.push((0x80 | ((codepoint >> 12) & 0x3F)) as u8);
        output_pointer.push((0x80 | ((codepoint >> 6) & 0x3F)) as u8);
        output_pointer.push((0x80 | (codepoint & 0x3F)) as u8);
    } else {
        return None;
    }

    Some(vec![0u8; seq_len])
}

