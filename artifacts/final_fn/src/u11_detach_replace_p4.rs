//@unit: U11_detach_replace_p4
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

pub fn replace_item_in_object(
    object: &mut Node,
    string: &[u8],
    new: Node,
) -> bool {
    replace_item_in_object_internal(object, string, new, false)
}

pub fn replace_item_in_object_case_sensitive(
    object: &mut Node,
    string: &[u8],
    new: Node,
) -> bool {
    replace_item_in_object_internal(object, string, new, true)
}

pub fn replace_item_in_object_internal(
    object: &mut Node,
    string: &[u8],
    mut new_item: Node,
    case_sensitive: bool,
) -> bool {
    let old = match get_object_item_internal(object, string, case_sensitive) {
        Some(item) => item.clone(),
        None => return false,
    };
    new_item.string = Some(string.to_vec());
    new_item.type_ &= !CJSON_STRING_IS_CONST;
    replace_item_via_pointer(object, &old, new_item)
}

pub use replace_item_in_object as cJSON_ReplaceItemInObject;
pub use replace_item_in_object_case_sensitive as cJSON_ReplaceItemInObjectCaseSensitive;
