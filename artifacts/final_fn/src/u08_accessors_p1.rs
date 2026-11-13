//@unit: U08_accessors_p1
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

pub fn get_array_item(array: Option<&Node>, index: i32) -> Option<&Node> {
    if index < 0 {
        return None;
    }
    let array = array?;
    get_array_item_internal(array, index as usize)
}

pub fn get_array_size(array: Option<&Node>) -> i32 {
    match array {
        Some(a) => a.children.len() as i32,
        None => 0,
    }
}

pub fn get_error_ptr() -> Option<&'static [u8]> {
    crate::contract::contract_get_error_ptr()
}

pub fn get_number_value(item: Option<&Node>) -> f64 {
    if item.map(|n| n.type_ & CJSON_NUMBER != 0).unwrap_or(false) {
        item.as_ref().unwrap().valuedouble
    } else {
        f64::NAN
    }
}

pub fn is_number(item: Option<&Node>) -> bool {
    item.map(|n| n.type_ & CJSON_NUMBER != 0).unwrap_or(false)
}

pub fn get_array_item_internal(array: &Node, index: usize) -> Option<&Node> {
    array.children.get(index)
}

pub use get_array_item as cJSON_GetArrayItem;
pub use get_array_size as cJSON_GetArraySize;
pub use get_error_ptr as cJSON_GetErrorPtr;
pub use get_number_value as cJSON_GetNumberValue;
pub use is_number as cJSON_IsNumber;
