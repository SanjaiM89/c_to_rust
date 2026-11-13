use crate::contract::*;

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
    let mut found = None;
    for child in &object.children {
        if child.string == string {
            found = Some(child.clone());
            break;
        }
    }

    if let Some(item) = found {
        object.children.retain(|child| child.string != string);
        Some(item)
    } else {
        None
    }
}

pub fn detach_item_from_object_case_sensitive(object: &mut Node, string: &[u8]) -> Option<Node> {
    let mut found_item = None;
    for child in &object.children {
        if child.string == string {
            found_item = Some(child.clone());
            break;
        }
    }

    if let Some(item) = found_item {
        object.children.retain(|child| child.string != string);
        Some(item)
    } else {
        None
    }
}
