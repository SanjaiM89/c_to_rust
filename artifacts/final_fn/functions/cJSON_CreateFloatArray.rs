pub fn create_float_array(numbers: &[f32], count: i32) -> Option<Node> {
    if count < 0 { return None; }
    let count = count as usize;
    let mut arr = create_array()?;
    for &num in numbers.iter().take(count) {
        let n = create_number(num as f64)?;
        arr.children.push(n);
    }
    Some(arr)
}
