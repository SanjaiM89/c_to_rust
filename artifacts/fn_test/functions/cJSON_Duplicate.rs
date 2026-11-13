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
