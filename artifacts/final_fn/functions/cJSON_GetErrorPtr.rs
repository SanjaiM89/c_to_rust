pub fn get_error_ptr() -> Option<&'static [u8]> {
    crate::contract::contract_get_error_ptr()
}
