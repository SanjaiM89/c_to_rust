// unit: U12_duplicate
pub fn duplicate(item: &Node, recursion: bool) -> Option<Node> {
    cjson_duplicate_rec(item, 0, recursion)
}

pub fn cjson_duplicate_rec(item: &Node, depth: usize, recursion: bool) -> Option<Node> {
    if depth >= CJSON_CIRCULAR_LIMIT {
        return None;
    }
    let mut new_item = Node::new(item.type_ & !CJSON_IS_REFERENCE);
    new_item.valueint = item.valueint;
    new_item.valuedouble = item.valuedouble;
    new_item.valuestring = item.valuestring.clone();
    new_item.string = item.string.clone();

    if recursion {
        for child in &item.children {
            let new_child = cjson_duplicate_rec(child, depth + 1, true)?;
            new_item.children.push(new_child);
        }
    }
    Some(new_item)
}
