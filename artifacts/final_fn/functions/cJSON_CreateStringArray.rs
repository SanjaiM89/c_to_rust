pub fn create_string_array(strings: &[&[u8]], count: i32) -> Option<Node> {
    if count < 0 { return None; }
    let count = count as usize;
    let mut arr = create_array()?;
    for &s in strings.iter().take(count) {
        let n = create_string(s)?;
        arr.children.push(n);
    }
    Some(arr)
}
