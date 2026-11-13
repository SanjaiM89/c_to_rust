pub fn get_array_size(array: Option<&Node>) -> i32 {
    match array {
        Some(a) => a.children.len() as i32,
        None => 0,
    }
}
