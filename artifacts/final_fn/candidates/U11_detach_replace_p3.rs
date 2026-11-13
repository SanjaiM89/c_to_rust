// unit: U11_detach_replace_p3
pub fn delete_item_from_object_case_sensitive(object: &mut Node, string: &[u8]) {
    detach_item_from_object_case_sensitive(object, string);
}

pub fn insert_item_in_array(array: &mut Node, which: i32, item: Node) -> bool {
    if which < 0 || item.children.is_empty() {
        return false;
    }

    let target_idx = which as usize;
    if target_idx >= array.children.len() {
        return false;
    }

    let target_node = &array.children[target_idx];
    if target_node.children.is_empty() || target_node.children.first().is_none() {
        return false;
    }

    let mut new_children = array.children.clone();
    let target_idx = new_children.iter().position(|n| n == target_node).unwrap();

    new_children.insert(target_idx + 1, item);

    array.children = new_children;
    true
}

pub fn replace_item_in_array(array: &mut Node, which: i32, new: Node) -> bool {
    if which < 0 {
        return false;
    }

    let index = which as usize;
    if index < array.children.len() {
        array.children[index] = new;
        return true;
    }

    false
}

pub fn replace_item_via_pointer(object: &mut Node, old: &Node, new: Node) -> bool {
    if object.children.is_empty() || old.children.is_empty() || new.children.is_empty() {
        return false;
    }

    if new == *old {
        return true;
    }

    let mut old_idx = 0;
    let mut found = false;

    for (idx, node) in object.children.iter().enumerate() {
        if node == old {
            old_idx = idx;
            found = true;
            break;
        }
    }

    if !found {
        return false;
    }

    // Replace the item in the children list
    object.children[old_idx] = new.clone();

    // Update neighbors if necessary
    let mut prev = None;
    let mut next = None;

    for (idx, node) in object.children.iter().enumerate() {
        if node == old {
            if idx > 0 {
                prev = Some(&object.children[idx - 1]);
            }
            if idx < object.children.len() - 1 {
                next = Some(&object.children[idx + 1]);
            }
        }
    }

    // Update the previous node's children list if needed
    if let Some(prev_node) = prev {
        for node in &prev_node.children {
            if node == &new {
                break;
            }
        }
    }

    // Update the next node's children list if needed
    if let Some(next_node) = next {
        if next_node == old {
            // This case is handled by the replacement
        }
    }

    // Remove the old node from the list
    object.children.remove(old_idx);

    true
}
