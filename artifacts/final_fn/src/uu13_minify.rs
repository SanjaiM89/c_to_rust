use crate::contract::*;

pub fn minify(json: &mut Vec<u8>) {
    let mut into = 0;
    let mut json_ptr = 0;

    while json_ptr < json.len() {
        match json[json_ptr] {
            b' ' | b'\t' | b'\r' | b'\n' => {
                json_ptr += 1;
            }
            b'/' => {
                if json_ptr + 1 < json.len() && json[json_ptr + 1] == b'/' {
                    // skip oneline comment
                    while json_ptr < json.len() && json[json_ptr] != b'\n' {
                        json_ptr += 1;
                    }
                } else if json_ptr + 1 < json.len() && json[json_ptr + 1] == b'*' {
                    // skip multiline comment
                    while json_ptr < json.len() && (json[json_ptr] != b'*' || json[json_ptr + 1] != b'/') {
                        json_ptr += 1;
                    }
                    if json_ptr < json.len() && json[json_ptr] == b'*' {
                        json_ptr += 1;
                    }
                } else {
                    json_ptr += 1;
                }
            }
            b'"' => {
                minify_string(json, &mut into, json_ptr);
                json_ptr += 1;
            }
            _ => {
                if into < json.len() {
                    into += 1;
                }
                json_ptr += 1;
            }
        }
    }

    // null-terminate
    if into < json.len() {
        json[into] = b'\0';
    }
}

pub fn minify_string(json: &[u8], into: &mut usize, start: usize) {
    let mut pos = start;
    let mut end = start;

    while pos < json.len() && json[pos] != b'"' {
        if json[pos] == b'\\' {
            pos += 1;
            if pos >= json.len() {
                break;
            }
        } else {
            end = pos + 1;
        }
        pos += 1;
    }

    if end > start {
        let slice = &json[start..end];
        let mut into_ptr = into;
        for &byte in slice {
            if into_ptr < json.len() {
                json[*into_ptr] = byte;
                *into_ptr += 1;
            }
        }
        *into = *into_ptr;
    }
}

pub fn minify_string(input: &mut &[u8], output: &mut Vec<u8>) {
    let mut input_iter = input.iter();
    let mut output_iter = output.iter_mut();

    // Skip the first character (it's always a quote)
    if input_iter.next().is_some() && input_iter.peekable().is_some() && input_iter.peekable().copied() == Some(b'"') {
        input_iter.next();
        output_iter.next();
    }

    loop {
        match input_iter.next() {
            Some(b'\"') => {
                output_iter.next();
                output_iter.next();
                return;
            }
            Some(b'\\') if input_iter.peekable().is_some() && input_iter.peekable().copied() == Some(b'"') => {
                output_iter.next();
                input_iter.next();
                input_iter.next();
                output_iter.next();
            }
            Some(b) => {
                output_iter.next();
                output_iter.next();
            }
            None => break,
        }
    }
}

pub fn skip_multiline_comment(input: &mut &[u8]) {
    let mut pos = 0;
    
    // Skip "/*"
    if pos + 2 >= input.len() {
        return;
    }
    
    pos += 2;
    
    loop {
        if pos >= input.len() {
            break;
        }
        
        let b0 = input[pos];
        let b1 = input[pos + 1];
        
        if b0 == b'*' && b1 == b'/' {
            pos += 2;
            break;
        }
        
        pos += 1;
    }
    
    *input = &input[pos..];
}

pub fn skip_oneline_comment(input: &mut &[u8]) {
    let comment_start = b"//";
    if input.starts_with(comment_start) {
        *input = &input[2..];
        for byte in input.iter() {
            if *byte == b'\n' {
                *input = &input[1..];
                return;
            }
        }
    }
}
