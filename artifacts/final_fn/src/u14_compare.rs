//@unit: U14_compare
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

pub fn compare(item1: Option<&Node>, item2: Option<&Node>, case_sensitive: bool) -> bool {
    let (a, b) = match (item1, item2) {
        (Some(a), Some(b)) => (a, b),
        (None, None) => return true,
        _ => return false,
    };

    if (a.type_ & 0xFF) != (b.type_ & 0xFF) {
        return false;
    }

    match a.type_ & 0xFF {
        CJSON_FALSE | CJSON_TRUE | CJSON_NULL => true,
        CJSON_NUMBER => compare_double(a.valuedouble, b.valuedouble),
        CJSON_STRING | CJSON_RAW => a.valuestring == b.valuestring,
        CJSON_ARRAY => {
            if a.children.len() != b.children.len() {
                return false;
            }
            for (ca, cb) in a.children.iter().zip(b.children.iter()) {
                if !compare(Some(ca), Some(cb), case_sensitive) {
                    return false;
                }
            }
            true
        }
        CJSON_OBJECT => {
            if a.children.len() != b.children.len() {
                return false;
            }
            for ca in &a.children {
                let name = match &ca.string {
                    Some(s) => s.as_slice(),
                    None => return false,
                };
                let cb = get_object_item_internal(b, name, case_sensitive);
                if !compare(Some(ca), cb, case_sensitive) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

pub use compare as cJSON_Compare;
