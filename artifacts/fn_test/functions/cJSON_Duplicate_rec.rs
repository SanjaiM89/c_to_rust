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
