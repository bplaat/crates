/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! String prototype method tests (Phase 3).
//! Test cases modeled after tc39/test262 test/built-ins/String/prototype/.

mod common;

use common::assert_js;
use js::Value;

fn s(v: &str) -> Value {
    Value::String(v.to_string())
}

#[test]
fn test_string_length() {
    assert_js(Value::Number(5.0), r#""hello".length"#);
    assert_js(Value::Number(0.0), r#""".length"#);
    assert_js(Value::Number(3.0), r#""abc".length"#);
}

#[test]
fn test_string_index_access() {
    assert_js(s("h"), r#""hello"[0]"#);
    assert_js(s("o"), r#""hello"[4]"#);
    assert_js(Value::Undefined, r#""hello"[99]"#);
}

#[test]
fn test_char_at() {
    assert_js(s("h"), r#""hello".charAt(0)"#);
    assert_js(s("e"), r#""hello".charAt(1)"#);
    assert_js(s(""), r#""hello".charAt(99)"#);
}

#[test]
fn test_char_code_at() {
    assert_js(Value::Number(72.0), r#""Hello".charCodeAt(0)"#);
    assert_js(Value::Number(101.0), r#""hello".charCodeAt(1)"#);
    assert_js(Value::Boolean(true), r#"isNaN("hello".charCodeAt(99))"#);
}

#[test]
fn test_index_of() {
    assert_js(Value::Number(1.0), r#""hello".indexOf("e")"#);
    assert_js(Value::Number(-1.0), r#""hello".indexOf("x")"#);
    assert_js(Value::Number(0.0), r#""hello".indexOf("hell")"#);
    assert_js(Value::Number(2.0), r#""abcabc".indexOf("c")"#);
    assert_js(Value::Number(5.0), r#""abcabc".indexOf("c", 3)"#);
}

#[test]
fn test_last_index_of() {
    assert_js(Value::Number(5.0), r#""abcabc".lastIndexOf("c")"#);
    assert_js(Value::Number(-1.0), r#""hello".lastIndexOf("x")"#);
    assert_js(Value::Number(2.0), r#""abcabc".lastIndexOf("c", 4)"#);
}

#[test]
fn test_slice() {
    assert_js(s("ello"), r#""hello".slice(1)"#);
    assert_js(s("ell"), r#""hello".slice(1, 4)"#);
    assert_js(s("lo"), r#""hello".slice(-2)"#);
    assert_js(s(""), r#""hello".slice(3, 1)"#);
}

#[test]
fn test_substring() {
    assert_js(s("ell"), r#""hello".substring(1, 4)"#);
    assert_js(s("ello"), r#""hello".substring(1)"#);
    // substring swaps start > end
    assert_js(s("ell"), r#""hello".substring(4, 1)"#);
}

#[test]
fn test_substr() {
    assert_js(s("ell"), r#""hello".substr(1, 3)"#);
    assert_js(s("ello"), r#""hello".substr(1)"#);
    assert_js(s("lo"), r#""hello".substr(-2)"#);
}

#[test]
fn test_to_upper_lower_case() {
    assert_js(s("HELLO"), r#""hello".toUpperCase()"#);
    assert_js(s("hello"), r#""HELLO".toLowerCase()"#);
    assert_js(
        s("Hello World"),
        r#""Hello World".toUpperCase().toLowerCase().charAt(0).toUpperCase() + "ello World""#,
    );
}

#[test]
fn test_trim() {
    assert_js(s("hello"), r#""  hello  ".trim()"#);
    assert_js(s("hello  "), r#""  hello  ".trimStart()"#);
    assert_js(s("  hello"), r#""  hello  ".trimEnd()"#);
    assert_js(s(""), r#""   ".trim()"#);
}

#[test]
fn test_split() {
    assert_js(
        Value::Array(js::ArrayValue {
            elements: std::rc::Rc::new(std::cell::RefCell::new(vec![s("a"), s("b"), s("c")])),
        }),
        r#""a,b,c".split(",")"#,
    );
    assert_js(Value::Number(3.0), r#""a,b,c".split(",").length"#);
    assert_js(Value::Number(1.0), r#""hello".split("x").length"#);
}

#[test]
fn test_replace() {
    assert_js(s("hXllo"), r#""hello".replace("e", "X")"#);
    assert_js(s("hello"), r#""hello".replace("x", "X")"#);
    // replaces only first occurrence
    assert_js(s("Xbcabc"), r#""abcabc".replace("a", "X")"#);
}

#[test]
fn test_replace_all() {
    assert_js(s("XbcXbc"), r#""abcabc".replaceAll("a", "X")"#);
    assert_js(s("hello"), r#""hello".replaceAll("x", "X")"#);
}

#[test]
fn test_concat() {
    assert_js(s("hello world"), r#""hello".concat(" world")"#);
    assert_js(s("abc"), r#""a".concat("b", "c")"#);
}

#[test]
fn test_includes() {
    assert_js(Value::Boolean(true), r#""hello world".includes("world")"#);
    assert_js(Value::Boolean(false), r#""hello world".includes("xyz")"#);
}

#[test]
fn test_starts_ends_with() {
    assert_js(Value::Boolean(true), r#""hello".startsWith("hell")"#);
    assert_js(Value::Boolean(false), r#""hello".startsWith("world")"#);
    assert_js(Value::Boolean(true), r#""hello".endsWith("llo")"#);
    assert_js(Value::Boolean(false), r#""hello".endsWith("hell")"#);
}

#[test]
fn test_repeat() {
    assert_js(s("abcabcabc"), r#""abc".repeat(3)"#);
    assert_js(s(""), r#""abc".repeat(0)"#);
}

#[test]
fn test_pad_start_end() {
    assert_js(s("00042"), r#""42".padStart(5, "0")"#);
    assert_js(s("  42"), r#""42".padStart(4)"#);
    assert_js(s("42000"), r#""42".padEnd(5, "0")"#);
    assert_js(s("42  "), r#""42".padEnd(4)"#);
}

#[test]
fn test_at() {
    assert_js(s("h"), r#""hello".at(0)"#);
    assert_js(s("o"), r#""hello".at(-1)"#);
    assert_js(s("l"), r#""hello".at(-2)"#);
    assert_js(Value::Undefined, r#""hello".at(99)"#);
}

#[test]
fn test_string_from_char_code() {
    assert_js(s("A"), r#"String.fromCharCode(65)"#);
    assert_js(s("ABC"), r#"String.fromCharCode(65, 66, 67)"#);
}
