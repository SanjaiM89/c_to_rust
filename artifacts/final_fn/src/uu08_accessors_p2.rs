use crate::contract::*;

pub fn get_object_item(object: Option<&Node>, string: &[u8]) -> Option<&Node> {
    object.as_ref().and_then(|node| {
        node.children
            .iter()
            .find(|item| item.string.as_ref() == string)
            .copied()
    })
}

pub fn get_object_item_case_sensitive(object: Option<&Node>, string: &[u8]) -> Option<&Node> {
    object.and_then(|obj| {
        obj.children
            .iter()
            .find(|node| node.type_ == CJSON_STRING && node.string.len() == string.len() && node.string.as_slice() == string)
            .copied()
    })
}

pub fn get_string_value(item: Option<&Node>) -> Option<&[u8]> {
    item.filter(|node| node.type_ == CJSON_STRING).map(|node| node.valuestring.as_ref())
}

pub fn is_string(item: Option<&Node>) -> bool {
    item.map(|n| n.type_ & CJSON_STRING == CJSON_STRING).unwrap_or(false)
}

pub fn case_insensitive_strcmp(string1: &[u8], string2: &[u8]) -> i32 {
    if string1 == string2 {
        return 0;
    }

    let mut i = 0;
    while i < string1.len() && i < string2.len() {
        let c1 = to_lower(string1[i]);
        let c2 = to_lower(string2[i]);
        if c1 != c2 {
            return (c1 as i32) - (c2 as i32);
        }
        i += 1;
    }

    if string1.len() == i && string2.len() == i {
        0
    } else if string1.len() < i {
        -1
    } else {
        1
    }
}

pub fn get_object_item(object: &Node, name: &[u8], case_sensitive: bool) -> Option<&Node> {
    let mut iter = object.children.iter();
    while let Some(node) = iter.next() {
        if case_sensitive {
            if node.string.is_none() || name != node.string.as_ref().as_ref() {
                continue;
            }
        } else {
            let name_lower = name.iter().map(|b| *b).map(|b| b).collect::<Vec<u8>>();
            let string_lower = node.string.as_ref().map(|s| {
                s.iter().map(|b| *b).map(|b| b).collect::<Vec<u8>>()
            }).unwrap_or_default();
            if node.string.is_none() || name_lower != string_lower {
                continue;
            }
        }
        return Some(node);
    }

    None
}
