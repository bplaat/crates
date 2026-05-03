/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! JSON built-in tests (Phase 8).
//! Test cases modeled after tc39/test262 test/built-ins/JSON/.

mod common;

use common::assert_js;
use js::Value;

fn n(v: f64) -> Value {
    Value::Number(v)
}

fn s(v: &str) -> Value {
    Value::String(v.to_string())
}

fn b(v: bool) -> Value {
    Value::Boolean(v)
}

// MARK: JSON.parse

#[test]
fn test_json_parse_primitives() {
    assert_js(n(42.0), r#"JSON.parse("42")"#);
    assert_js(n(1.5), r#"JSON.parse("1.5")"#);
    assert_js(s("hello"), r#"JSON.parse('"hello"')"#);
    assert_js(b(true), r#"JSON.parse("true")"#);
    assert_js(b(false), r#"JSON.parse("false")"#);
    assert_js(Value::Null, r#"JSON.parse("null")"#);
}

#[test]
fn test_json_parse_object() {
    assert_js(n(42.0), r#"JSON.parse('{"a":42}').a"#);
    assert_js(s("hi"), r#"JSON.parse('{"x":"hi"}').x"#);
}

#[test]
fn test_json_parse_array() {
    assert_js(n(1.0), r#"JSON.parse("[1,2,3]")[0]"#);
    assert_js(n(3.0), r#"JSON.parse("[1,2,3]").length"#);
}

#[test]
fn test_json_parse_nested() {
    assert_js(n(99.0), r#"JSON.parse('{"a":{"b":99}}').a.b"#);
}

// MARK: JSON.stringify

#[test]
fn test_json_stringify_primitives() {
    assert_js(s("42"), r#"JSON.stringify(42)"#);
    assert_js(s("3.14"), r#"JSON.stringify(3.14)"#);
    assert_js(s(r#""hello""#), r#"JSON.stringify("hello")"#);
    assert_js(s("true"), r#"JSON.stringify(true)"#);
    assert_js(s("false"), r#"JSON.stringify(false)"#);
    assert_js(s("null"), r#"JSON.stringify(null)"#);
}

#[test]
fn test_json_stringify_object() {
    assert_js(s(r#"{"a":1}"#), r#"JSON.stringify({a: 1})"#);
    assert_js(s(r#"{"x":"hi"}"#), r#"JSON.stringify({x: "hi"})"#);
}

#[test]
fn test_json_stringify_array() {
    assert_js(s("[1,2,3]"), r#"JSON.stringify([1, 2, 3])"#);
    assert_js(s(r#"["a","b"]"#), r#"JSON.stringify(["a", "b"])"#);
}

#[test]
fn test_json_roundtrip() {
    // Parse then stringify should be idempotent for simple values
    assert_js(
        s(r#"{"a":1,"b":2}"#),
        r#"JSON.stringify(JSON.parse('{"a":1,"b":2}'))"#,
    );
    assert_js(s("[1,2,3]"), r#"JSON.stringify(JSON.parse("[1,2,3]"))"#);
}
