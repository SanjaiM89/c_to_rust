use crate::contract::*;

pub fn create_string_array(strings: &[&[u8]], count: i32) -> Option<Node> {
    if count < 0 || strings.is_empty() {
        return None;
    }

    let mut array = Node {
        children: Vec::new(),
    };

    for string in strings {
        let node = Node {
            children: vec![],
            string: string.to_vec(),
            type_: 0,
            valuedouble: 0.0,
            valueint: 0,
            valuestring: string.to_vec(),
        };
        array.children.push(node);
    }

    Some(array)
}

pub fn create_string_reference(string: &[u8]) -> Option<Node> {
    let node = Node {
        type_: CJSON_STRING | CJSON_IS_REFERENCE,
        valuestring: Some(string.to_vec()),
        children: Vec::new(),
        string: Some(string.to_vec()),
        valuedouble: None,
    };
    Some(node)
}

pub fn create_true() -> Option<Node> {
    Some(Node {
        type_: CJSON_TRUE,
        children: Vec::new(),
        string: None,
        valuedouble: None,
        valueint: None,
    })
}

pub fn set_valuestring(object: &mut Node, valuestring: &[u8]) -> Option<Vec<u8>> {
    if object.type_ & CJSON_STRING == 0 || (object.type_ & CJSON_IS_REFERENCE) != 0 {
        return None;
    }
    if object.valuestring.is_none() || valuestring.is_empty() {
        return None;
    }

    let v1_len = valuestring.len();
    let v2_len = object.valuestring.as_ref().map(|s| s.len()).unwrap_or(0);

    // Check for overlapping strings
    if v1_len <= v2_len {
        let valuestring_ptr = valuestring.as_ptr() as usize;
        let object_valuestring_ptr = object.valuestring.as_ref().map(|s| s.as_ptr() as usize).unwrap_or(0);

        if !(valuestring_ptr.add(v1_len) < object_valuestring_ptr ||
             object_valuestring_ptr.add(v2_len) < valuestring_ptr) {
            return None;
        }
        object.valuestring.as_mut().unwrap().copy_from_slice(valuestring);
        return Some(object.valuestring.take().unwrap());
    }

    let copy = valuestring.to_vec();
    if let Some(old) = object.valuestring.take() {
        // No free function, so just drop old value (no memory management)
    }
    object.valuestring = Some(copy);
    Some(copy)
}

pub fn create_reference(item: &Node, hooks: &Hooks) -> Option<Node> {
    if item.is_none() {
        return None;
    }

    let reference = hooks.alloc(std::mem::size_of::<Node>());
    if reference.is_none() {
        return None;
    }

    let mut reference = reference.unwrap();
    reference.type_ = item.type_;
    reference.valueint = item.valueint;
    reference.valuedouble = item.valuedouble;
    reference.string = None;
    reference.type_ |= CJSON_IS_REFERENCE;

    Some(reference)
}
