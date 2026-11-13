pub fn set_number_helper(object: &mut Node, number: f64) -> f64 {
    if number >= i32::MAX as f64 {
        object.valueint = i32::MAX;
    } else if number <= (i32::MIN as f64) {
        object.valueint = i32::MIN;
    } else {
        object.valueint = (number as i32);
    }

    object.valuedouble = number;
    number
}
