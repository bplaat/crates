/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Error type tests (Phase 9).
//! Test cases modeled after tc39/test262 test/built-ins/Error/.

mod common;

use common::assert_js;
use js::Value;

fn s(v: &str) -> Value {
    Value::String(v.to_string())
}

#[test]
fn test_error_basic() {
    assert_js(s("Error"), r#"Error("boom").name"#);
    assert_js(s("boom"), r#"Error("boom").message"#);
    assert_js(s("Error: boom"), r#"Error("boom").stack"#);
}

#[test]
fn test_error_empty_message() {
    assert_js(s("Error"), r#"Error().name"#);
    assert_js(s(""), r#"Error().message"#);
}

#[test]
fn test_type_error() {
    assert_js(s("TypeError"), r#"TypeError("bad type").name"#);
    assert_js(s("bad type"), r#"TypeError("bad type").message"#);
}

#[test]
fn test_range_error() {
    assert_js(s("RangeError"), r#"RangeError("out of range").name"#);
    assert_js(s("out of range"), r#"RangeError("out of range").message"#);
}

#[test]
fn test_syntax_error() {
    assert_js(s("SyntaxError"), r#"SyntaxError("bad syntax").name"#);
}

#[test]
fn test_reference_error() {
    assert_js(s("ReferenceError"), r#"ReferenceError("not defined").name"#);
}

#[test]
fn test_eval_error() {
    assert_js(s("EvalError"), r#"EvalError("eval error").name"#);
}

#[test]
fn test_uri_error() {
    assert_js(s("URIError"), r#"URIError("bad uri").name"#);
}

#[test]
fn test_error_throw_catch() {
    assert_js(
        s("boom"),
        r#"
        let msg = "";
        try {
            throw Error("boom");
        } catch(e) {
            msg = e.message;
        }
        msg
    "#,
    );
}

#[test]
fn test_typed_error_throw_catch() {
    assert_js(
        s("TypeError"),
        r#"
        let name = "";
        try {
            throw TypeError("wrong type");
        } catch(e) {
            name = e.name;
        }
        name
    "#,
    );
}
