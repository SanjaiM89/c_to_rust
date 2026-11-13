pub fn init_hooks(hooks: Option<&Hooks>) {
    GLOBAL_HOOKS.with(|h| {
        let mut gh = h.borrow_mut();
        if let Some(h_in) = hooks {
            *gh = Hooks::default();
            if h_in.allocate.is_some() {
                gh.allocate = h_in.allocate;
            }
            if h_in.deallocate.is_some() {
                gh.deallocate = h_in.deallocate;
            }
            if h_in.reallocate.is_some() {
                gh.reallocate = h_in.reallocate;
            }
        } else {
            *gh = Hooks::default();
        }
    });
}
