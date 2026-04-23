/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Global object and builtin function tests.

mod common;

use common::assert_js;
use js::Value;

#[test]
fn test_globals() {
    assert_js(Value::Boolean(true), "isNaN(NaN)");
    assert_js(Value::Boolean(false), "isNaN(42)");
    assert_js(Value::Boolean(true), "isNaN('string')");
    assert_js(Value::Boolean(true), "isNaN(0 / 0)");
    assert_js(Value::Boolean(true), "isFinite(42)");
    assert_js(Value::Boolean(false), "isFinite(Infinity)");
    assert_js(Value::Boolean(false), "isFinite(NaN)");
    assert_js(Value::Boolean(false), "isFinite(1 / 0)");
}

#[test]
fn test_globalthis_access() {
    assert_js(
        Value::Boolean(true),
        "let obj = { getGlobal() { return typeof globalThis === 'object'; } }; obj.getGlobal()",
    );
}

#[test]
fn test_globalthis_global_variables() {
    assert_js(
        Value::Number(42.0),
        "globalVar = 42; let obj = { getVar() { return globalThis.globalVar; } }; obj.getVar()",
    );
}

#[test]
fn test_globalthis_readonly_properties() {
    assert_js(
        Value::Boolean(true),
        "let obj = { check() { return typeof globalThis.Infinity === 'number'; } }; obj.check()",
    );
}

#[test]
fn test_globalthis_multiple_accesses() {
    assert_js(
        Value::Number(20.0),
        "globalX = 10; let obj = { getValue() { return globalThis.globalX * 2; } }; obj.getValue()",
    );
}

#[test]
fn test_globalthis_same_across_methods() {
    assert_js(
        Value::Boolean(true),
        "globalTest = 123; let obj = { m1() { return globalThis.globalTest; }, m2() { return globalThis.globalTest === 123; } }; obj.m2()",
    );
}

#[test]
fn test_globalthis_change() {
    assert_js(
        Value::Boolean(true),
        "globalThis = 123; let obj = { m2() { return globalThis === 123; } }; obj.m2()",
    );
}
