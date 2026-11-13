// differential_rust.rs - reference driver for the Rust implementation.
//
// Must produce a byte-identical transcript to harness/differential_c.c for the
// same input. Any difference is a divergence between the C and the Rust.
//
// The transcript covers: parse success, error offset, the parsed tree
// (canonical, byte-exact), formatted and unformatted printing, minifying,
// and a print/parse round trip.

use std::io::{Read, Write};

use cjson_rs::*;

fn print_hex(out: &mut Vec<u8>, bytes: &[u8]) {
    for b in bytes {
        out.extend_from_slice(format!("{:02x}", b).as_bytes());
    }
}

fn format_d_c(d: f64) -> String {
    let mut buf = [0u8; 64];
    extern "C" {
        fn snprintf(buf: *mut u8, size: usize, fmt: *const u8, ...) -> i32;
    }
    unsafe {
        let n = snprintf(buf.as_mut_ptr(), buf.len(), b"%.17g\0".as_ptr(), d);
        String::from_utf8_lossy(&buf[..n as usize]).to_string()
    }
}

fn print_tree(out: &mut Vec<u8>, item: Option<&Node>, depth: usize) {
    for _ in 0..depth {
        out.push(b' ');
    }
    out.extend_from_slice(b"N");
    match item {
        None => {
            out.extend_from_slice(b" t=-1\n");
        }
        Some(node) => {
            out.extend_from_slice(format!(" t={}", node.type_).as_bytes());
            out.extend_from_slice(format!(" i={}", node.valueint).as_bytes());
            out.extend_from_slice(format!(" d={}", format_d_c(node.valuedouble)).as_bytes());
            out.extend_from_slice(b" k=");
            print_hex(out, node.string.as_deref().unwrap_or(b""));
            out.extend_from_slice(b" v=");
            print_hex(out, node.valuestring.as_deref().unwrap_or(b""));
            out.push(b'\n');
            for child in node.children.iter() {
                print_tree(out, Some(child), depth + 1);
            }
        }
    }
}

fn main() {
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input).unwrap();
    let mut out: Vec<u8> = Vec::new();

    out.extend_from_slice(format!("INPUT_LEN {}\n", input.len()).as_bytes());

    // ---- parse ----
    let mut parse_end: usize = 0;
    let root = parse_with_opts(&input, Some(&mut parse_end), false);

    let root = match root {
        Some(r) => r,
        None => {
            out.extend_from_slice(b"STATUS fail\n");
            let off = get_error_position().map(|p| p as i64).unwrap_or(-1);
            out.extend_from_slice(format!("ERROR_OFFSET {}\n", off).as_bytes());
            out.extend_from_slice(b"TREE\n");
            out.extend_from_slice(b"PRINT\n");
            out.extend_from_slice(b"PRINT_UNFORMATTED\n");
            out.extend_from_slice(b"MINIFY\n");
            out.extend_from_slice(b"ROUNDTRIP none\n");
            std::io::stdout().write_all(&out).unwrap();
            return;
        }
    };
    out.extend_from_slice(b"STATUS ok\n");
    out.extend_from_slice(format!("ERROR_OFFSET {}\n", parse_end).as_bytes());

    // ---- tree ----
    out.extend_from_slice(b"TREE\n");
    print_tree(&mut out, Some(&root), 0);

    // ---- printing ----
    let printed = print(&root);
    let printed_unf = print_unformatted(&root);
    out.extend_from_slice(b"PRINT ");
    match &printed {
        Some(p) => out.extend_from_slice(p),
        None => out.extend_from_slice(b"(null)"),
    }
    out.push(b'\n');
    out.extend_from_slice(b"PRINT_UNFORMATTED ");
    match &printed_unf {
        Some(p) => out.extend_from_slice(p),
        None => out.extend_from_slice(b"(null)"),
    }
    out.push(b'\n');

    // ---- minify (on a copy) ----
    out.extend_from_slice(b"MINIFY ");
    let mut copy = input.clone();
    minify(&mut copy);
    out.extend_from_slice(&copy);
    out.push(b'\n');

    // ---- round trip ----
    match &printed_unf {
        Some(p) => {
            match parse(p) {
                Some(_) => {
                    out.extend_from_slice(b"ROUNDTRIP ok\n");
                    let again = print_unformatted(&parse(p).unwrap());
                    let equal = match &again {
                        Some(a) => a == p,
                        None => false,
                    };
                    out.extend_from_slice(
                        format!("ROUNDTRIP_EQUAL {}\n", if equal { "yes" } else { "no" }).as_bytes(),
                    );
                }
                None => {
                    out.extend_from_slice(b"ROUNDTRIP fail\n");
                    out.extend_from_slice(b"ROUNDTRIP_EQUAL no\n");
                }
            }
        }
        None => {
            out.extend_from_slice(b"ROUNDTRIP none\n");
            out.extend_from_slice(b"ROUNDTRIP_EQUAL no\n");
        }
    }

    std::io::stdout().write_all(&out).unwrap();
}
