use crate::contract::*;

pub fn duplicate(item: &Node, recursion: bool) -> Option<Node> {
    if item.is_none() {
        return None;
    }
    let mut newitem = new__item(&Hooks::default())?;
    newitem.type_ = item.type_ & !CJSON_IS_REFERENCE;
    newitem.valueint = item.valueint;
    newitem.valuedouble = item.valuedouble;

    if let Some(valuestring) = &item.valuestring {
        if let Some(string) = cjson_strdup(valuestring, &Hooks::default()) {
            newitem.valuestring = Some(string);
        } else {
            return None;
        }
    }

    if let Some(string) = &item.string {
        if item.type_ & CJSON_STRING_IS_CONST != 0 {
            newitem.string = Some(string.to_vec());
        } else {
            if let Some(string) = cjson_strdup(string, &Hooks::default()) {
                newitem.string = Some(string);
            } else {
                return None;
            }
        }
    }

    if !recursion {
        return Some(newitem);
    }

    let mut children = Vec::new();
    let mut next = None;
    let mut child = &item.children;

    for node in child {
        if depth >= CJSON_CIRCULAR_LIMIT {
            return None;
        }
        let newchild = duplicate(node, true)?;
        if let Some(ref mut next_node) = next {
            next_node.next = Some(newchild.clone());
            newchild.prev = Some(next_node.clone());
            next = Some(newchild.clone());
        } else {
            newitem.children = Some(vec![newchild]);
            next = Some(newchild.clone());
        }
        children.push(newchild);
    }

    if let Some(ref mut first_child) = newitem.children {
        if let Some(ref last_child) = children.last() {
            first_child.last_mut().unwrap().prev = Some(last_child.clone());
        }
    }

    Some(newitem)
}

pub fn cjson_duplicate_rec(item: &Node, depth: usize, recursion: bool) -> Option<Node> {
    if item.is_none() {
        return None;
    }
    let mut newitem = new__item(&Hooks::default())?;
    newitem.type_ = item.type_ & !CJSON_IS_REFERENCE;
    newitem.valueint = item.valueint;
    newitem.valuedouble = item.valuedouble;

    if let Some(valuestring) = &item.valuestring {
        if let Some(string) = cjson_strdup(valuestring, &Hooks::default()) {
            newitem.valuestring = Some(string);
        } else {
            return None;
        }
    }

    if let Some(string) = &item.string {
        if item.type_ & CJSON_STRING_IS_CONST != 0 {
            newitem.string = Some(string.to_vec());
        } else {
            if let Some(string) = cjson_strdup(string, &Hooks::default()) {
                newitem.string = Some(string);
            } else {
                return None;
            }
        }
    }

    if !recursion {
        return Some(newitem);
    }

    let mut children = Vec::new();
    let mut next = None;
    let mut child = &item.children;

    for node in child {
        if depth >= CJSON_CIRCULAR_LIMIT {
            return None;
        }
        let newchild = cjson_duplicate_rec(node, depth + 1, true)?;
        if let Some(ref mut next_node) = next {
            next_node.next = Some(newchild.clone());
            newchild.prev = Some(next_node.clone());
            next = Some(newchild.clone());
        } else {
            newitem.children = Some(vec![newchild]);
            next = Some(newchild.clone());
        }
        children.push(newchild);
    }

    if let Some(ref mut first_child) = newitem.children {
        if let Some(ref last_child) = children.last() {
            first_child.last_mut().unwrap().prev = Some(last_child.clone());
        }
    }

    Some(newitem)
}
