pub fn get_number_value(item: Option<&Node>) -> f64 {
    if item.map(|n| n.type_ & CJSON_NUMBER != 0).unwrap_or(false) {
        item.as_ref().unwrap().valuedouble
    } else {
        f64::NAN
    }
}
