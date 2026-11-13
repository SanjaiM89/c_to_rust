//@unit: U08_accessors_p2
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

pub fn get_object_item<'a>(object: Option<&'a Node>, string: &[u8]) -> Option<&'a Node> {
    let obj = object?;
    get_object_item_internal(obj, string, false)
}

pub fn get_object_item_case_sensitive<'a>(
    object: Option<&'a Node>,
    string: &[u8],
) -> Option<&'a Node> {
    let obj = object?;
    get_object_item_internal(obj, string, true)
}

pub fn get_string_value(item: Option<&Node>) -> Option<&[u8]> {
    let item = item?;
    if !is_string(Some(item)) {
        return None;
    }
    item.valuestring.as_deref()
}

pub fn is_string(item: Option<&Node>) -> bool {
    item.map(|n| n.type_ & CJSON_STRING == CJSON_STRING).unwrap_or(false)
}

pub fn case_insensitive_strcmp(string1: &[u8], string2: &[u8]) -> i32 {
    if string1 == string2 {
        return 0;
    }

    let mut i = 0;
    while i < string1.len() && i < string2.len() {
        let c1 = to_lower(string1[i]);
        let c2 = to_lower(string2[i]);
        if c1 != c2 {
            return (c1 as i32) - (c2 as i32);
        }
        i += 1;
    }

    if string1.len() == i && string2.len() == i {
        0
    } else if string1.len() < i {
        -1
    } else {
        1
    }
}

pub fn get_object_item_internal<'a>(
    object: &'a Node,
    name: &[u8],
    case_sensitive: bool,
) -> Option<&'a Node> {
    for child in &object.children {
        if let Some(s) = &child.string {
            if case_sensitive {
                if s.as_slice() == name {
                    return Some(child);
                }
            } else {
                if case_insensitive_strcmp(name, s.as_slice()) == 0 {
                    return Some(child);
                }
            }
        }
    }
    None
}

pub use get_object_item as cJSON_GetObjectItem;
pub use get_object_item_case_sensitive as cJSON_GetObjectItemCaseSensitive;
pub use get_string_value as cJSON_GetStringValue;
pub use is_string as cJSON_IsString;
