/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Object method and computed property tests.

mod common;

use common::assert_js;
use js::Value;

#[test]
fn test_this_in_regular_functions() {
    assert_js(
        Value::Undefined,
        "let obj = { val: 100, getValue() { return this.val; } }; let func = obj.getValue; func()",
    );
}

#[test]
fn test_this_in_nested_methods() {
    assert_js(
        Value::Number(20.0),
        "let obj = { x: 10, nested: { y: 20, getY() { return this.y; } } }; obj.nested.getY()",
    );
}

#[test]
fn test_this_in_called_method() {
    assert_js(
        Value::Number(100.0),
        "let obj = { val: 100, getValue() { return this.val; } }; obj.getValue()",
    );
}

#[test]
fn test_this_object_creation_pattern() {
    assert_js(
        Value::Number(15.0),
        "let obj = { x: 10, y: 5, init(a, b) { this.x = a; this.y = b; return this.x + this.y; } }; obj.init(7, 8)",
    );
}

#[test]
fn test_this_in_chained_methods() {
    assert_js(
        Value::Number(100.0),
        "let obj = { val: 50, double() { this.val = this.val * 2; return this.val; } }; obj.double()",
    );
}

#[test]
fn test_method_shorthand() {
    assert_js(
        Value::Number(10.0),
        "let obj = { getValue() { return 10; } }; obj.getValue()",
    );
    assert_js(
        Value::Number(15.0),
        "let obj = { add(a, b) { return a + b; } }; obj.add(7, 8)",
    );
    assert_js(
        Value::Number(3.0),
        "let obj = { getValue() { return 10; }, getThree() { return 3; } }; obj.getThree()",
    );
    assert_js(
        Value::String(String::from("hello")),
        "let obj = { greet() { return 'hello'; } }; obj.greet()",
    );
    assert_js(
        Value::Number(5.0),
        "let obj = { x: 10, double() { return this.x / 2; } }; obj.double()",
    );
}

#[test]
fn test_computed_properties() {
    assert_js(
        Value::Number(100.0),
        "let key = 'prop'; let obj = { [key]: 100 }; obj.prop",
    );
    assert_js(Value::Number(42.0), "let obj = { ['x']: 42 }; obj.x");
    assert_js(
        Value::String(String::from("dynamic")),
        "let obj = { [1 + 1]: 'dynamic' }; obj['2']",
    );
    assert_js(
        Value::Number(200.0),
        "let key1 = 'a'; let key2 = 'b'; let obj = { [key1]: 100, [key2]: 200 }; obj.b",
    );
    assert_js(
        Value::Number(55.0),
        "let obj = { ['num']: 55 }; let k = 'num'; obj[k]",
    );
}

#[test]
fn test_method_shorthand_with_objects() {
    assert_js(
        Value::Number(25.0),
        "let obj = { x: 10, y: 15, sum() { return this.x + this.y; } }; obj.sum()",
    );
    assert_js(
        Value::String(String::from("hello world")),
        "let obj = { greeting: 'hello', name: 'world', greet() { return this.greeting + ' ' + this.name; } }; obj.greet()",
    );
}

#[test]
fn test_mixed_shorthand_and_computed() {
    assert_js(
        Value::Number(30.0),
        "let key = 'compute'; let obj = { x: 10, [key]: 20, getValue() { return this.x + this.compute; } }; obj.getValue()",
    );
    assert_js(
        Value::Number(30.0),
        "let obj = { a: 10, [1 + 1]: 20, sum() { return this.a + this['2']; } }; obj.sum()",
    );
}
