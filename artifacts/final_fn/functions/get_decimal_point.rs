pub fn get_decimal_point() -> u8 {
    // In Rust, we don't have locale support via localeconv directly,
    // and the C function uses localeconv which is not available in standard Rust.
    // Since the C code falls back to '.' when locales are disabled,
    // and there's no equivalent locale configuration in Rust without external crates,
    // we default to the fallback behavior.
    b'.'
}
