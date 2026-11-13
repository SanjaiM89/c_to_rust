//@unit: U10_add_item_p6
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

pub fn add_true_to_object(object: &mut Node, name: &[u8]) -> Option<Node> {
    let true_item = create_true()?;
    let copy = true_item.clone();
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        if add_item_to_object_internal(object, name, true_item, &gh, false) {
            Some(copy)
        } else {
            None
        }
    })
}

pub use add_true_to_object as cJSON_AddTrueToObject;
