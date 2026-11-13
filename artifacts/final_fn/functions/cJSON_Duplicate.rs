pub fn duplicate(item: &Node, recursion: bool) -> Option<Node> {
    cjson_duplicate_rec(item, 0, recursion)
}
