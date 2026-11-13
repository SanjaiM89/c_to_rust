//@unit: U11_detach_replace_p2
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

pub fn delete_item_from_object(object: &mut Node, string: &[u8]) {
    let mut children = object.children.clone();
    let mut index = 0;
    while index < children.len() {
        if let Some(child_string) = &children[index].string {
            if child_string.as_slice() == string {
                children.remove(index);
                break;
            }
        }
        index += 1;
    }
    object.children = children;
}

pub fn detach_item_from_object(object: &mut Node, string: &[u8]) -> Option<Node> {
    let to_detach = get_object_item(Some(&*object), string)?.clone();
    detach_item_via_pointer(object, &to_detach)
}

pub fn detach_item_from_object_case_sensitive(object: &mut Node, string: &[u8]) -> Option<Node> {
    let pos = object.children.iter().position(|child| {
        child.string.as_deref().map_or(false, |s| s == string)
    })?;
    Some(object.children.remove(pos))
}

pub use delete_item_from_object as cJSON_DeleteItemFromObject;
pub use detach_item_from_object as cJSON_DetachItemFromObject;
pub use detach_item_from_object_case_sensitive as cJSON_DetachItemFromObjectCaseSensitive;
