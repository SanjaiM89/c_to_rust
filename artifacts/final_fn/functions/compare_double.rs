pub fn compare_double(a: f64, b: f64) -> bool {
    let max_val = (a.abs().max(b.abs()));
    (a - b).abs() <= max_val * f64::EPSILON
}
