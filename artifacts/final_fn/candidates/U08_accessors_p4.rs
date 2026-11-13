// unit: U08_accessors_p4
pub fn is_object(item: Option<&Node>) -> bool {
    item.map(|node| node.type_ & CJSON_OBJECT != 0).unwrap_or(false)
}

pub fn is_raw(item: Option<&Node>) -> bool {
    item.map(|node| node.type_ & CJSON_RAW == CJSON_RAW).unwrap_or(false)
}

pub fn is_true(item: Option<&Node>) -> bool {
    item.map(|n| n.type_ & CJSON_TRUE != 0).unwrap_or(false)
}

pub fn version() -> &'static str {
    static VERSION: &str = "1.0.0";
    VERSION
}
