//! Rust-native conformance tests.
//!
//! These are ports of the assertions in the upstream cJSON Unity test suite
//! (`vendor/cJSON/tests/*.c`). The upstream suite is C and links against the C
//! library; these are the same expectations expressed against the Rust API.
//!
//! Expected values are copied from the upstream tests verbatim, e.g.
//! `parse_number.c` asserts `assert_parse_number("0.001", 0, 0.001)` and
//! `print_number.c` asserts `assert_print_number("1e-09", 10e-10)`.

use cjson_rs::*;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn assert_is_number(n: &Node) {
    assert!(n.children.is_empty(), "number must have no children");
    assert_eq!(n.type_ & 0xFF, CJSON_NUMBER, "not a number");
    assert_eq!(n.type_ & CJSON_IS_REFERENCE, 0, "must not be a reference");
    assert_eq!(n.type_ & CJSON_STRING_IS_CONST, 0, "must not be a const string");
    assert!(n.valuestring.is_none(), "number must have no valuestring");
    assert!(n.string.is_none(), "number must have no key");
}

fn assert_parse_number(s: &str, integer: i32, real: f64) {
    let mut item = Node::new(CJSON_NUMBER);
    let mut buf = ParseBuffer::new(s.as_bytes());
    assert!(
        cjson_rs::u03_number_parse::parse_number(&mut item, &mut buf),
        "parse_number failed for {s:?}"
    );
    assert_is_number(&item);
    assert_eq!(item.valueint, integer, "valueint for {s:?}");
    assert_eq!(item.valuedouble, real, "valuedouble for {s:?}");
}

fn assert_parse_big_number(s: &str) {
    let mut item = Node::new(CJSON_NUMBER);
    let mut buf = ParseBuffer::new(s.as_bytes());
    assert!(
        cjson_rs::u03_number_parse::parse_number(&mut item, &mut buf),
        "parse_number failed for {s:?}"
    );
    assert_is_number(&item);
}

fn assert_print_number(expected: &str, input: f64) {
    let mut item = Node::new(CJSON_NUMBER);
    item.valuedouble = input;
    item.valueint = input as i32;
    let mut buf = PrintBuffer::default();
    buf.buffer = vec![0u8; 1024];
    buf.length = 1024;
    buf.noalloc = true;
    assert!(
        cjson_rs::u07_value_print_p1::print_number(&item, &mut buf),
        "print_number failed"
    );
    let got = String::from_utf8_lossy(&buf.buffer[..buf.offset]).to_string();
    assert_eq!(got, expected, "printed number");
}

// ---------------------------------------------------------------------------
// parse_hex4 (upstream: parse_hex4.c)
// ---------------------------------------------------------------------------
#[test]
fn parse_hex4_should_parse_all_combinations() {
    for number in 0u32..=0xFFFF {
        let lower = format!("{number:04x}");
        let upper = format!("{number:04X}");
        assert_eq!(
            cjson_rs::u04_string_parse::parse_hex4(lower.as_bytes()),
            number,
            "lowercase {lower}"
        );
        assert_eq!(
            cjson_rs::u04_string_parse::parse_hex4(upper.as_bytes()),
            number,
            "uppercase {upper}"
        );
    }
}

#[test]
fn parse_hex4_should_parse_mixed_case() {
    for s in [
        "beef", "beeF", "beEf", "beEF", "bEef", "bEeF", "bEEf", "bEEF", "Beef", "BeeF", "BeEf",
        "BeEF", "BEef", "BEeF", "BEEf", "BEEF",
    ] {
        assert_eq!(
            cjson_rs::u04_string_parse::parse_hex4(s.as_bytes()),
            0xBEEF,
            "mixed case {s}"
        );
    }
}

#[test]
fn parse_hex4_should_fail_on_invalid_input() {
    // upstream asserts these are 0
    for s in ["", "x", "0x1", "zzzz", "00"] {
        assert_eq!(
            cjson_rs::u04_string_parse::parse_hex4(s.as_bytes()),
            0,
            "invalid input {s:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// parse_number (upstream: parse_number.c)
// ---------------------------------------------------------------------------
#[test]
fn parse_number_should_parse_zero() {
    assert_parse_number("0", 0, 0.0);
    assert_parse_number("0.0", 0, 0.0);
    assert_parse_number("-0", 0, -0.0);
}

#[test]
fn parse_number_should_parse_negative_integers() {
    assert_parse_number("-1", -1, -1.0);
    assert_parse_number("-32768", -32768, -32768.0);
    assert_parse_number("-2147483648", i32::MIN, -2147483648.0);
}

#[test]
fn parse_number_should_parse_positive_integers() {
    assert_parse_number("1", 1, 1.0);
    assert_parse_number("32767", 32767, 32767.0);
    assert_parse_number("2147483647", i32::MAX, 2147483647.0);
}

#[test]
fn parse_number_should_parse_positive_reals() {
    assert_parse_number("0.001", 0, 0.001);
    assert_parse_number("10e-10", 0, 10e-10);
    assert_parse_number("10E-10", 0, 10e-10);
    assert_parse_number("10e10", i32::MAX, 10e10);
    assert_parse_number("123e+127", i32::MAX, 123e127);
    assert_parse_number("123e-128", 0, 123e-128);
}

#[test]
fn parse_number_should_parse_negative_reals() {
    assert_parse_number("-0.001", 0, -0.001);
    assert_parse_number("-10e-10", 0, -10e-10);
    assert_parse_number("-10E-10", 0, -10e-10);
    assert_parse_number("-10e20", i32::MIN, -10e20);
}

#[test]
fn parse_number_should_parse_big_numbers() {
    assert_parse_big_number("123456789012345678901234567890");
    assert_parse_big_number("-123456789012345678901234567890");
    assert_parse_big_number("1e999");
    assert_parse_big_number("1e-999");
}

// ---------------------------------------------------------------------------
// print_number (upstream: print_number.c)
// ---------------------------------------------------------------------------
#[test]
fn print_number_should_print_zero() {
    assert_print_number("0", 0.0);
}

#[test]
fn print_number_should_print_negative_integers() {
    assert_print_number("-1", -1.0);
    assert_print_number("-32768", -32768.0);
    assert_print_number("-2147483648", -2147483648.0);
}

#[test]
fn print_number_should_print_positive_integers() {
    assert_print_number("1", 1.0);
    assert_print_number("32767", 32767.0);
    assert_print_number("2147483647", 2147483647.0);
}

#[test]
fn print_number_should_print_positive_reals() {
    assert_print_number("0.123", 0.123);
    assert_print_number("1e-09", 10e-10);
    assert_print_number("1000000000000", 10e11);
    assert_print_number("1.23e+129", 123e+127);
    assert_print_number("1.23e-126", 123e-128);
    assert_print_number("3.1415926535897931", 3.1415926535897931);
}

#[test]
fn print_number_should_print_negative_reals() {
    assert_print_number("-0.0123", -0.0123);
    assert_print_number("-1e-09", -10e-10);
    assert_print_number("-1e+21", -10e20);
    assert_print_number("-1.23e+129", -123e+127);
    assert_print_number("-1.23e-126", -123e-128);
}

// ---------------------------------------------------------------------------
// parsing / printing round trips (upstream: parse_*.c, print_*.c)
// ---------------------------------------------------------------------------
fn printed(s: &str) -> String {
    let node = parse(s.as_bytes()).unwrap_or_else(|| panic!("parse failed for {s:?}"));
    let out = print_unformatted(&node).unwrap_or_else(|| panic!("print failed for {s:?}"));
    String::from_utf8_lossy(&out).to_string()
}

#[test]
fn parse_should_handle_basic_values() {
    assert_eq!(printed("null"), "null");
    assert_eq!(printed("true"), "true");
    assert_eq!(printed("false"), "false");
    assert_eq!(printed("0"), "0");
    assert_eq!(printed("\"hello\""), "\"hello\"");
    assert_eq!(printed("[]"), "[]");
    assert_eq!(printed("{}"), "{}");
}

#[test]
fn parse_should_handle_containers() {
    assert_eq!(printed("[1,2,3]"), "[1,2,3]");
    assert_eq!(printed("{\"a\":1}"), "{\"a\":1}");
    assert_eq!(
        printed("{\"a\":[1,2,{\"b\":null}]}"),
        "{\"a\":[1,2,{\"b\":null}]}"
    );
}

#[test]
fn parse_should_reject_invalid_documents() {
    for s in [
        "", "{", "}", "[", "]", "{\"a\"}", "{\"a\":}", "{a:1}", "[1,]", "tru", "nul",
        "\"unterminated", "{\"a\":1,}",
    ] {
        assert!(
            parse(s.as_bytes()).is_none(),
            "expected {s:?} to be rejected"
        );
    }
}

#[test]
fn parse_should_preserve_raw_bytes() {
    // cJSON accepts invalid UTF-8; a Rust String would not.
    let mut input = Vec::from(&b"\"a\xffb\""[..]);
    let node = parse(&input).unwrap();
    assert_eq!(node.valuestring.as_deref(), Some(&b"a\xffb"[..]));
    let _ = &mut input;
}

#[test]
fn parse_should_reject_deeply_nested_input() {
    // cJSON_NESTING_LIMIT is 1000
    let deep = format!("{}{}", "[".repeat(1200), "]".repeat(1200));
    assert!(parse(deep.as_bytes()).is_none(), "deep nesting must be rejected");
    let ok = format!("{}{}", "[".repeat(100), "]".repeat(100));
    assert!(parse(ok.as_bytes()).is_some(), "100 levels must parse");
}

#[test]
fn print_should_escape_strings() {
    let node = parse(b"\"a\\\"b\\\\c\\nd\"").unwrap();
    let out = print_unformatted(&node).unwrap();
    assert_eq!(String::from_utf8_lossy(&out), "\"a\\\"b\\\\c\\nd\"");
}

// ---------------------------------------------------------------------------
// accessors (upstream: misc_tests.c)
// ---------------------------------------------------------------------------
#[test]
fn accessors_should_reject_missing_items() {
    assert!(cjson_rs::u08_accessors_p1::cJSON_GetArraySize(None).eq(&0));
    assert!(cjson_rs::u08_accessors_p1::cJSON_GetArrayItem(None, 0).is_none());
    assert!(cjson_rs::u08_accessors_p2::cJSON_GetObjectItem(None, b"x").is_none());
    assert!(cjson_rs::u08_accessors_p3::cJSON_HasObjectItem(None, b"x") == false);
    assert!(cjson_rs::u08_accessors_p3::cJSON_IsInvalid(None) == false);
    assert!(cjson_rs::u08_accessors_p3::cJSON_IsNull(None) == false);
}

#[test]
fn accessors_should_find_items() {
    let node = parse(b"{\"key\":[10,20,30]}").unwrap();
    let arr = cjson_rs::u08_accessors_p2::cJSON_GetObjectItem(Some(&node), b"key").unwrap();
    assert_eq!(cjson_rs::u08_accessors_p1::cJSON_GetArraySize(Some(arr)), 3);
    let first = cjson_rs::u08_accessors_p1::cJSON_GetArrayItem(Some(arr), 0).unwrap();
    assert_eq!(cjson_rs::u08_accessors_p1::cJSON_GetNumberValue(Some(first)), 10.0);
    assert!(cjson_rs::u08_accessors_p1::cJSON_GetArrayItem(Some(arr), 3).is_none());
}

#[test]
fn object_lookup_is_case_insensitive_by_default() {
    let node = parse(b"{\"Key\":1}").unwrap();
    assert!(
        cjson_rs::u08_accessors_p2::cJSON_GetObjectItem(Some(&node), b"key").is_some(),
        "default lookup is case insensitive"
    );
    assert!(
        cjson_rs::u08_accessors_p2::cJSON_GetObjectItemCaseSensitive(Some(&node), b"key")
            .is_none(),
        "case sensitive lookup must not match"
    );
}

// ---------------------------------------------------------------------------
// creating and adding items (upstream: cjson_add.c)
// ---------------------------------------------------------------------------
#[test]
fn create_should_produce_typed_nodes() {
    assert_eq!(cjson_rs::u09_create_p3::cJSON_CreateNull().unwrap().type_, CJSON_NULL);
    assert_eq!(cjson_rs::u09_create_p4::cJSON_CreateTrue().unwrap().type_, CJSON_TRUE);
    assert_eq!(cjson_rs::u09_create_p2::cJSON_CreateFalse().unwrap().type_, CJSON_FALSE);
    assert_eq!(
        cjson_rs::u09_create_p1::cJSON_CreateNumber(42.0).unwrap().valuedouble,
        42.0
    );
    assert_eq!(
        cjson_rs::u09_create_p3::cJSON_CreateString(b"hi").unwrap().valuestring,
        Some(b"hi".to_vec())
    );
    assert!(cjson_rs::u09_create_p1::cJSON_CreateArray().unwrap().children.is_empty());
    assert!(cjson_rs::u09_create_p3::cJSON_CreateObject().unwrap().children.is_empty());
}

#[test]
fn add_should_append_to_objects_and_arrays() {
    let mut obj = cjson_rs::u09_create_p3::cJSON_CreateObject().unwrap();
    cjson_rs::u10_add_item_p4::cJSON_AddNullToObject(&mut obj, b"null");
    cjson_rs::u10_add_item_p4::cJSON_AddNumberToObject(&mut obj, b"number", 42.0);
    cjson_rs::u10_add_item_p5::cJSON_AddStringToObject(&mut obj, b"string", b"value");
    assert_eq!(obj.children.len(), 3);
    let out = print_unformatted(&obj).unwrap();
    assert_eq!(
        String::from_utf8_lossy(&out),
        "{\"null\":null,\"number\":42,\"string\":\"value\"}"
    );
}

#[test]
fn add_should_fail_when_the_allocator_fails() {
    // upstream: cjson_add.c installs a failing allocator and expects NULL
    fn failing(_size: usize) -> Option<Vec<u8>> {
        None
    }
    let hooks = Hooks {
        allocate: Some(failing),
        deallocate: None,
        reallocate: None,
    };
    init_hooks(Some(&hooks));
    let mut obj = cjson_rs::u09_create_p3::cJSON_CreateObject().unwrap();
    let r = cjson_rs::u10_add_item_p4::cJSON_AddNullToObject(&mut obj, b"null");
    init_hooks(None);
    assert!(r.is_none(), "allocation failure must be reported");
}

// ---------------------------------------------------------------------------
// minify (upstream: minify_tests.c)
// ---------------------------------------------------------------------------
#[test]
fn minify_should_remove_whitespace_and_comments() {
    let mut s = Vec::from(&b"{\n\t\"a\" : 1 , /* c */ \"b\" : 2 // line\n}"[..]);
    minify(&mut s);
    assert_eq!(String::from_utf8_lossy(&s), "{\"a\":1,\"b\":2}");
}

// ---------------------------------------------------------------------------
// duplicate (upstream: misc_tests.c)
// ---------------------------------------------------------------------------
#[test]
fn duplicate_should_copy_the_tree() {
    let node = parse(b"{\"a\":[1,2,3],\"b\":\"x\"}").unwrap();
    let copy = cjson_rs::u12_duplicate::cJSON_Duplicate(&node, true).unwrap();
    let a = print_unformatted(&node).unwrap();
    let b = print_unformatted(&copy).unwrap();
    assert_eq!(a, b, "duplicate must print identically");
}

// ---------------------------------------------------------------------------
// error reporting (upstream: misc_tests.c)
// ---------------------------------------------------------------------------
#[test]
fn parse_error_position_is_reported() {
    assert!(parse(b"{invalid}").is_none());
    assert!(get_error_position().is_some(), "an error position must be set");
}
