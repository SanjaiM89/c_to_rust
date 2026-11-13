pub fn parse_hex4(input: &[u8]) -> u32 {
    if input.len() < 4 {
        return 0;
    }
    let mut h = 0u32;
    for i in 0..4 {
        let b = input[i];
        if (b >= b'0' && b <= b'9') {
            h += (b - b'0') as u32;
        } else if (b >= b'A' && b <= b'F') {
            h += (10 + b - b'A') as u32;
        } else if (b >= b'a' && b <= b'f') {
            h += (10 + b - b'a') as u32;
        } else {
            return 0;
        }

        if i < 3 {
            h <<= 4;
        }
    }
    h
}
