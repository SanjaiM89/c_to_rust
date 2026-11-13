use crate::contract::*;

pub fn compare(item1: Option<&Node>, item2: Option<&Node>, case_sensitive: bool) -> bool {
    if item1.is_none() || item2.is_none() {
        return false;
    }

    let type1 = item1.as_ref().type_;
    let type2 = item2.as_ref().type_;

    if type1 != type2 {
        return false;
    }

    // Identical objects are equal
    if item1 == item2 {
        return true;
    }

    match type1 {
        CJSON_FALSE | CJSON_TRUE | CJSON_NULL => true,
        CJSON_NUMBER => {
            let val1 = item1.as_ref().valuedouble;
            let val2 = item2.as_ref().valuedouble;
            compare_double(val1, val2)
        }
        CJSON_STRING | CJSON_RAW => {
            let s1 = item1.as_ref().string;
            let s2 = item2.as_ref().string;
            if s1.is_none() || s2.is_none() {
                return false;
            }
            if case_sensitive {
                s1 == s2
            } else {
                let s1_lower = s1.iter().map(|b| to_lower(*b)).collect::<Vec<u8>>();
                let s2_lower = s2.iter().map(|b| to_lower(*b)).collect::<Vec<u8>>();
                s1_lower == s2_lower
            }
        }
        CJSON_ARRAY => {
            let mut a_iter = item1.as_ref().children.iter();
            let mut b_iter = item2.as_ref().children.iter();
            while let Some(a_node) = a_iter.next() {
                if let Some(b_node) = b_iter.next() {
                    if !compare(Some(a_node), Some(b_node), case_sensitive) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            a_iter.next().is_none() && b_iter.next().is_none()
        }
        CJSON_OBJECT => {
            let mut a_iter = item1.as_ref().children.iter();
            let mut b_iter = item2.as_ref().children.iter();

            // Check all keys in a
            while let Some(a_node) = a_iter.next() {
                if let Some(b_node) = get_object_item(item2.as_ref(), a_node.string.as_ref(), case_sensitive) {
                    if !compare(Some(a_node), Some(b_node), case_sensitive) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Check all keys in b
            while let Some(b_node) = b_iter.next() {
                if let Some(a_node) = get_object_item(item1.as_ref(), b_node.string.as_ref(), case_sensitive) {
                    if !compare(Some(b_node), Some(a_node), case_sensitive) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        }
        _ => false,
    }
}
