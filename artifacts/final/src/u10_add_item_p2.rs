//@unit: U10_add_item_p2
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

pub fn add_array_to_object(object: &mut Node, name: &[u8]) -> Option<Node> {
    let array = create_array()?;
    let copy = array.clone();
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        if add_item_to_object_internal(object, name, array, &gh, false) {
            Some(copy)
        } else {
            None
        }
    })
}

pub fn add_bool_to_object(object: &mut Node, name: &[u8], boolean: bool) -> Option<Node> {
    let bool_item = create_bool(boolean)?;
    let copy = bool_item.clone();
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        if add_item_to_object_internal(object, name, bool_item, &gh, false) {
            Some(copy)
        } else {
            None
        }
    })
}

pub use add_array_to_object as cJSON_AddArrayToObject;
pub use add_bool_to_object as cJSON_AddBoolToObject;
