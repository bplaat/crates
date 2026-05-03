/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Object built-in tests (Phase 7).
//! Test cases modeled after tc39/test262 test/built-ins/Object/.

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

fn arr(v: Vec<Value>) -> Value {
    Value::Array(js::ArrayValue {
        elements: std::rc::Rc::new(std::cell::RefCell::new(v)),
    })
}

#[test]
fn test_object_keys() {
    assert_js(arr(vec![s("a"), s("b")]), r#"Object.keys({a: 1, b: 2})"#);
    assert_js(arr(vec![]), r#"Object.keys({})"#);
}

#[test]
fn test_object_values() {
    assert_js(arr(vec![n(1.0), n(2.0)]), r#"Object.values({a: 1, b: 2})"#);
}

#[test]
fn test_object_entries() {
    assert_js(
        arr(vec![arr(vec![s("a"), n(1.0)]), arr(vec![s("b"), n(2.0)])]),
        r#"Object.entries({a: 1, b: 2})"#,
    );
}

#[test]
fn test_object_assign() {
    assert_js(
        n(2.0),
        r#"let o = {a:1}; Object.assign(o, {a:2, b:3}); o.a"#,
    );
    assert_js(
        n(3.0),
        r#"let o = {a:1}; Object.assign(o, {a:2, b:3}); o.b"#,
    );
}

#[test]
fn test_object_assign_multiple_sources() {
    assert_js(
        n(3.0),
        r#"let o = {}; Object.assign(o, {a:1}, {b:2}, {c:3}); o.c"#,
    );
}

#[test]
fn test_object_from_entries() {
    assert_js(n(1.0), r#"Object.fromEntries([["a", 1], ["b", 2]]).a"#);
    assert_js(n(2.0), r#"Object.fromEntries([["a", 1], ["b", 2]]).b"#);
}

#[test]
fn test_object_get_own_property_names() {
    let result = arr(vec![s("a"), s("b")]);
    assert_js(result, r#"Object.getOwnPropertyNames({a:1, b:2})"#);
}

#[test]
fn test_object_has_own() {
    assert_js(b(true), r#"Object.hasOwn({a: 1}, "a")"#);
    assert_js(b(false), r#"Object.hasOwn({a: 1}, "b")"#);
}

#[test]
fn test_object_create() {
    // Object.create copies properties from proto (simplified, no real prototype chain yet)
    assert_js(
        n(1.0),
        r#"let proto = {x: 1}; let obj = Object.create(proto); obj.x"#,
    );
}

#[test]
fn test_object_freeze() {
    // Object.freeze returns the object (further writes are silently ignored)
    assert_js(n(1.0), r#"let o = {a:1}; Object.freeze(o); o.a = 99; o.a"#);
}
