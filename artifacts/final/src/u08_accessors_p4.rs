//@unit: U08_accessors_p4
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

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

pub use is_object as cJSON_IsObject;
pub use is_raw as cJSON_IsRaw;
pub use is_true as cJSON_IsTrue;
pub use version as cJSON_Version;
