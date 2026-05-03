/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Number built-in tests (Phase 6).
//! Test cases modeled after tc39/test262 test/built-ins/Number/.

mod common;

use common::assert_js;
use js::Value;

fn n(v: f64) -> Value {
    Value::Number(v)
}

fn b(v: bool) -> Value {
    Value::Boolean(v)
}

fn s(v: &str) -> Value {
    Value::String(v.to_string())
}

#[test]
fn test_number_constants() {
    assert_js(Value::Boolean(true), "Number.isFinite(Number.MAX_VALUE)");
    assert_js(Value::Boolean(true), "Number.isFinite(Number.MIN_VALUE)");
    assert_js(
        Value::Boolean(true),
        "Number.POSITIVE_INFINITY === Infinity",
    );
    assert_js(
        Value::Boolean(true),
        "Number.NEGATIVE_INFINITY === -Infinity",
    );
    assert_js(Value::Boolean(true), "Number.isNaN(Number.NaN)");
    assert_js(
        Value::Boolean(true),
        "Number.MAX_SAFE_INTEGER === 9007199254740991",
    );
    assert_js(
        Value::Boolean(true),
        "Number.MIN_SAFE_INTEGER === -9007199254740991",
    );
}

#[test]
fn test_number_is_nan() {
    assert_js(b(true), "Number.isNaN(NaN)");
    assert_js(b(false), "Number.isNaN(1)");
    assert_js(b(false), "Number.isNaN(\"NaN\")"); // does NOT coerce (unlike global isNaN)
    assert_js(b(false), "Number.isNaN(undefined)");
}

#[test]
fn test_number_is_finite() {
    assert_js(b(true), "Number.isFinite(1)");
    assert_js(b(false), "Number.isFinite(Infinity)");
    assert_js(b(false), "Number.isFinite(-Infinity)");
    assert_js(b(false), "Number.isFinite(NaN)");
    assert_js(b(false), "Number.isFinite(\"1\")"); // does NOT coerce
}

#[test]
fn test_number_is_integer() {
    assert_js(b(true), "Number.isInteger(1)");
    assert_js(b(true), "Number.isInteger(0)");
    assert_js(b(false), "Number.isInteger(1.5)");
    assert_js(b(false), "Number.isInteger(NaN)");
    assert_js(b(false), "Number.isInteger(Infinity)");
}

#[test]
fn test_number_to_fixed() {
    assert_js(s("3.14"), "(3.14159).toFixed(2)");
    assert_js(s("3"), "(3.14159).toFixed(0)");
    assert_js(s("3.0"), "(3.0).toFixed(1)");
    assert_js(s("1.50"), "(1.5).toFixed(2)");
}

#[test]
fn test_number_to_string() {
    assert_js(s("42"), "(42).toString()");
    assert_js(s("101010"), "(42).toString(2)"); // binary
    assert_js(s("2a"), "(42).toString(16)"); // hex
    assert_js(s("52"), "(42).toString(8)"); // octal
}

#[test]
fn test_number_coercion() {
    assert_js(n(42.0), "Number(\"42\")");
    assert_js(n(0.0), "Number(\"\")");
    assert_js(n(0.0), "Number(null)");
    assert_js(n(0.0), "Number(false)");
    assert_js(n(1.0), "Number(true)");
    assert_js(Value::Boolean(true), "isNaN(Number(undefined))");
    assert_js(Value::Boolean(true), "isNaN(Number(\"abc\"))");
}

#[test]
fn test_number_parse() {
    assert_js(n(42.0), "Number.parseInt(\"42\")");
    assert_js(n(42.0), "Number.parseFloat(\"42.0\")");
}
