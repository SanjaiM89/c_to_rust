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
