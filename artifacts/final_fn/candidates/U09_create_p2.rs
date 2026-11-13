// unit: U09_create_p2
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

pub fn create_false() -> Option<Node> {
    Some(Node::new(CJSON_FALSE))
}

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

pub fn create_int_array(numbers: &[i32], count: i32) -> Option<Node> {
    if count < 0 { return None; }
    let count = count as usize;
    let mut arr = create_array()?;
    for &num in numbers.iter().take(count) {
        let n = create_number(num as f64)?;
        arr.children.push(n);
    }
    Some(arr)
}

pub fn suffix_object(prev: &mut Node, item: Node) {
    prev.children.push(item);
}
