//@unit: U10_add_item_p1
#![allow(unused_imports, unused_variables, dead_code)]
use crate::contract::*;
use crate::*;

pub fn add_item_to_array_internal(array: &mut Node, item: Node) -> bool {
    array.children.push(item);
    true
}

pub fn add_item_to_object_internal(
    object: &mut Node,
    string: &[u8],
    mut item: Node,
    hooks: &Hooks,
    constant_key: bool,
) -> bool {
    let new_key = if constant_key {
        cast_away_const(&string).to_vec()
    } else {
        match cjson_strdup(string, hooks) {
            Some(k) => k,
            None => return false,
        }
    };

    item.string = Some(new_key);
    if constant_key {
        item.type_ |= CJSON_STRING_IS_CONST;
    } else {
        item.type_ &= !CJSON_STRING_IS_CONST;
    }

    add_item_to_array_internal(object, item)
}

