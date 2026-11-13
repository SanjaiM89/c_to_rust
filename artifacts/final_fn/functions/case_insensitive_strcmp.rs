pub fn case_insensitive_strcmp(string1: &[u8], string2: &[u8]) -> i32 {
    if string1 == string2 {
        return 0;
    }

    let mut i = 0;
    while i < string1.len() && i < string2.len() {
        let c1 = to_lower(string1[i]);
        let c2 = to_lower(string2[i]);
        if c1 != c2 {
            return (c1 as i32) - (c2 as i32);
        }
        i += 1;
    }

    if string1.len() == i && string2.len() == i {
        0
    } else if string1.len() < i {
        -1
    } else {
        1
    }
}
