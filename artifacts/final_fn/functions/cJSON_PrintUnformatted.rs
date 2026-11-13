pub fn print_unformatted(item: &Node) -> Option<Vec<u8>> {
    GLOBAL_HOOKS.with(|h| {
        let gh = h.borrow();
        print_internal(item, false, &gh)
    })
}
