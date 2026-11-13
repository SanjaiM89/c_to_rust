pub fn cjson_strdup(string: &[u8], hooks: &Hooks) -> Option<Vec<u8>> {
    if string.is_empty() {
        return None;
    }

    let s = if let Some(pos) = string.iter().position(|&b| b == 0) {
        &string[..pos]
    } else {
        string
    };

    let mut copy = hooks.alloc(s.len())?;
    copy.copy_from_slice(s);
    Some(copy)
}
