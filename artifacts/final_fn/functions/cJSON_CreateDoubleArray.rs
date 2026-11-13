pub fn create_double_array(numbers: &[f64], count: i32) -> Option<Node> {
    if count < 0 { return None; }
    let count = count as usize;
    let mut arr = create_array()?;
    for &num in numbers.iter().take(count) {
        let n = create_number(num)?;
        arr.children.push(n);
    }
    Some(arr)
}
