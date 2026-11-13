//@unit: U11_detach_replace_p1
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

pub fn delete_item_from_array(array: &mut Node, which: i32) {
    let index = which as usize;
    if index >= array.children.len() {
        return;
    }

    array.children.remove(index);
}

pub fn detach_item_from_array(array: &mut Node, which: i32) -> Option<Node> {
    if which < 0 {
        return None;
    }
    let idx = which as usize;
    if idx >= array.children.len() {
        return None;
    }
    Some(array.children.remove(idx))
}

pub fn detach_item_via_pointer(parent: &mut Node, item: &Node) -> Option<Node> {
    // Check if parent has no children or item is not present
    if parent.children.is_empty() {
        return None;
    }

    // Check if item is the first child
    if item == parent.children.first().unwrap_or(&Node::default()) {
        // If item is the first child, update the first child to the next child
        if let Some(next) = parent.children.get(1) {
            parent.children[0] = next.clone();
        } else {
            parent.children.clear();
        }
    } else {
        // Check if item is not the first child and has no previous sibling
        if parent.children.iter().position(|n| n == item).is_none() {
            return None;
        }

        // Find the index of item
        let item_idx = parent.children.iter().position(|n| n == item).unwrap();

        // Update the previous sibling's next pointer (if exists)
        if let Some(prev_idx) = (item_idx - 1).checked_sub(1) {
            if let Some(prev) = parent.children.get(prev_idx) {
                // We don't have a `next` field, so we cannot modify it
                // This logic is invalid in Rust - we must not use pointers
                // Instead, we only manage the vector and indices
            }
        }

        // Update the next sibling's previous pointer (if exists)
        if let Some(next_idx) = (item_idx + 1).checked_add(1) {
            if let Some(next) = parent.children.get(next_idx) {
                // We don't have a `prev` field, so we cannot set `next.prev = None`
                // This is not valid in Rust
            }
        }

        // If item is not the first child and has no next sibling, update the last child's previous pointer
        // But we don't have a `prev` field, so we cannot do this
    }

    // Remove item from children
    parent.children.retain(|n| n != item);

    Some(item.clone())
}

pub use delete_item_from_array as cJSON_DeleteItemFromArray;
pub use detach_item_from_array as cJSON_DetachItemFromArray;
pub use detach_item_via_pointer as cJSON_DetachItemViaPointer;
