// model: Qwen3-4B-Instruct-2507-Q4_K_M (function granularity)
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
    if item.is_none() {
        return None;
    }

    let mut newitem = Node::new();
    newitem.type_ = item.type_ & (!CJSON_IS_REFERENCE);
    newitem.valueint = item.valueint;
    newitem.valuedouble = item.valuedouble;

    if let Some(s) = &item.valuestring {
        let s = s.to_vec();
        newitem.valuestring = Some(s);
    }

    if let Some(s) = &item.string {
        let s = if item.type_ & CJSON_STRING_IS_CONST != 0 {
            s.to_vec()
        } else {
            s.to_vec()
        };
        newitem.string = Some(s);
    }

    if !recursion {
        return Some(newitem);
    }

    let mut child = item.children().next();
    while let Some(child) = child {
        if depth >= CJSON_CIRCULAR_LIMIT {
            return None;
        }

        let newchild = cjson_duplicate_rec(child, depth + 1, true);
        if newchild.is_none() {
            return None;
        }

        let next = child.children().next();
        if let Some(next) = next {
            if let Some(prev) = next.prev() {
                prev.next = Some(newchild.clone());
                newchild.as_mut().unwrap().prev = Some(prev);
            } else {
                newitem.children.push(newchild.unwrap());
            }
        } else {
            newitem.children.push(newchild.unwrap());
        }

        child = child.children().next();
    }

    if let Some(first) = newitem.children.first() {
        first.prev = Some(newitem.children.last().unwrap());
    }

    Some(newitem)
}
