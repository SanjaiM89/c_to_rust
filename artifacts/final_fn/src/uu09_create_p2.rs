use crate::contract::*;

pub fn create_double_array(numbers: &[f64], count: i32) -> Option<Node> {
    if count < 0 || numbers.is_empty() {
        return None;
    }

    let mut array = Node {
        children: Vec::new(),
        string: None,
        type_: 0,
        valuedouble: 0.0,
        valueint: 0,
        valuestring: None,
    };

    for (i, &num) in numbers.iter().enumerate() {
        let node = Node {
            children: Vec::new(),
            string: None,
            type_: 0,
            valuedouble: 0.0,
            valueint: 0,
            valuestring: None,
        };

        array.children.push(node);
    }

    Some(array)
}

pub fn create_false() -> Option<Node> {
    Some(Node {
        type_: CJSON_FALSE,
        children: Vec::new(),
        string: None,
        valuedouble: 0.0,
        valueint: 0,
    })
}

pub fn create_float_array(numbers: &[f32], count: i32) -> Option<Node> {
    if count < 0 || numbers.is_empty() {
        return None;
    }

    let mut array = Node {
        children: Vec::new(),
    };

    for (i, &num) in numbers.iter().enumerate() {
        let node = Node {
            children: Vec::new(),
        };

        if i == 0 {
            array.children.push(node);
        } else {
            let current_last = array.children.last_mut();
            if let Some(last) = current_last {
                last.children.push(node);
            } else {
                array.children.push(node);
            }
        }
    }

    Some(array)
}

pub fn create_int_array(numbers: &[i32], count: i32) -> Option<Node> {
    if count < 0 || numbers.is_empty() {
        return None;
    }

    let mut array = Node {
        valueint: None,
        valuestring: None,
        type_: None,
        valuedouble: None,
        children: Vec::new(),
    };

    for (i, &num) in numbers.iter().enumerate() {
        let node = Node {
            valueint: Some(num),
            valuestring: None,
            type_: Some(1),
            valuedouble: None,
            children: Vec::new(),
        };

        if i == 0 {
            array.children.push(node);
        } else {
            array.children.push(node);
        }
    }

    Some(array)
}

pub fn suffix_object(prev: &mut Node, item: Node) {
    // In Rust, there is no `next` or `prev` field on `Node`.
    // The C function `suffix_object` attempts to link `prev->next = item` and `item->prev = prev`.
    // However, in the Rust model, `children` is a `Vec<Node>`, and there is no `next` or `prev` field.
    // Therefore, this function does nothing because the C linkage semantics do not exist in Rust.
    // The behavior of the C function is not representable in the Rust contract as defined.
}
