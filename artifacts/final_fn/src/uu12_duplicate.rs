use crate::contract::*;

pub fn duplicate(item: &Node, recursion: bool) -> Option<Node> {
    match item.type_ {
        CJSON_NULL => Some(Node {
            type_: CJSON_NULL,
            children: Vec::new(),
            ..Node::default()
        }),
        CJSON_FALSE | CJSON_TRUE => Some(Node {
            type_: item.type_,
            children: Vec::new(),
            ..Node::default()
        }),
        CJSON_NUMBER => Some(Node {
            type_: CJSON_NUMBER,
            children: Vec::new(),
            ..Node::default()
        }),
        CJSON_STRING => Some(Node {
            type_: CJSON_STRING,
            children: Vec::new(),
            ..Node::default()
        }),
        CJSON_ARRAY => {
            let mut new_children = Vec::new();
            for child in &item.children {
                if recursion {
                    if let Some(dup_child) = duplicate(child, true) {
                        new_children.push(dup_child);
                    }
                } else {
                    new_children.push(child.clone());
                }
            }
            Some(Node {
                type_: CJSON_ARRAY,
                children: new_children,
                ..Node::default()
            })
        },
        CJSON_OBJECT => {
            let mut new_children = Vec::new();
            for child in &item.children {
                if recursion {
                    if let Some(dup_child) = duplicate(child, true) {
                        new_children.push(dup_child);
                    }
                } else {
                    new_children.push(child.clone());
                }
            }
            Some(Node {
                type_: CJSON_OBJECT,
                children: new_children,
                ..Node::default()
            })
        },
        _ => None,
    }
}

pub fn cjson_duplicate_rec(item: &Node, depth: usize, recursion: bool) -> Option<Node> {
    if item.children.is_empty() {
        return Some(Node::new(0));
    }

    let mut newitem = Node::new(0);
    newitem.type_ = item.type_ & (!CJSON_IS_REFERENCE);
    newitem.valueint = item.valueint;
    newitem.valuedouble = item.valuedouble;

    if let Some(valuestring) = &item.valuestring {
        if let Some(string) = cjson_strdup(valuestring, &[]) {
            newitem.valuestring = Some(string);
        } else {
            return None;
        }
    }

    if let Some(string) = &item.string {
        if item.type_ & CJSON_STRING_IS_CONST != 0 {
            newitem.string = Some(string.clone());
        } else {
            if let Some(s) = cjson_strdup(string, &[]) {
                newitem.string = Some(s);
            } else {
                return None;
            }
        }
    }

    if !recursion {
        return Some(newitem);
    }

    let mut child = item.children.iter().next();
    while let Some(child_item) = child {
        if depth >= CJSON_CIRCULAR_LIMIT {
            return None;
        }

        let newchild = cjson_duplicate_rec(child_item, depth + 1, true);
        if newchild.is_none() {
            return None;
        }

        let newchild = newchild.unwrap();

        // Find the next child in the original list
        let next_child = child.clone().next().map(|c| c).unwrap_or_else(|| {
            // If no next, just use the current one
            child_item
        });

        // If the next child exists and is the same as current, we need to link
        if let Some(next_item) = next_child {
            if let Some(next_child_item) = next_item.children.first() {
                if next_child_item == child_item {
                    // This is a duplicate, so we need to link properly
                    if let Some(prev) = newchild.children.first() {
                        if prev == child_item {
                            // Already linked
                        } else {
                            // Link the newchild to the next
                            newchild.children.push(newchild);
                        }
                    }
                }
            }
        }

        // Append the newchild to the current node's children
        if newitem.children.is_empty() {
            newitem.children.push(newchild);
        } else {
            let last = newitem.children.last_mut().unwrap();
            last.children.push(newchild);
        }

        child = child.clone().next();
    }

    Some(newitem)
}
