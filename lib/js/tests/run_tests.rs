/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Run JS test assertions.

#![cfg(test)]

use js::{Context, Value};

fn assert_js(expected: Value, script: &str) {
    let mut context = Context::new();
    let result = context.eval(script).unwrap();
    assert_eq!(expected, result);
}

#[test]
fn test_keywords() {
    assert_js(Value::Undefined, "undefined");
    assert_js(Value::Null, "null");
    assert_js(Value::Boolean(true), "true");
    assert_js(Value::Boolean(false), "false");
}

#[test]
fn test_arithmetic() {
    assert_js(Value::Number(0), "0");
    assert_js(Value::Number(42), "42");
    assert_js(Value::Number(21), "5+20-4");
    assert_js(Value::Number(41), " 12 + 34 - 5 ");
    assert_js(Value::Number(47), "5+6*7");
    assert_js(Value::Number(15), "5*(9-6)");
    assert_js(Value::Number(4), "(3 + 5) / 2");
    assert_js(Value::Number(16), "2 ** 4");
    assert_js(Value::Number(0), "36 / 0");
    assert_js(Value::Number(2), "20 % 3");
    assert_js(Value::Number(34), "--34");
    assert_js(Value::Number(-40), "---40");
    assert_js(Value::Number(1), "5 & 3");
    assert_js(Value::Number(7), "5 | 3");
    assert_js(Value::Number(6), "5 ^ 3");
    assert_js(Value::Number(16), "8 << 1");
    assert_js(Value::Number(4), "8 >> 1");
    assert_js(Value::Number(0), "8 >> 4");
    assert_js(Value::Number(4), "8 >>> 1");
    assert_js(Value::Number(0), "8 >>> 4");
}

#[test]
fn test_comparisons() {
    assert_js(Value::Boolean(true), "5 < 10");
    assert_js(Value::Boolean(false), "5 < 3");
    assert_js(Value::Boolean(true), "5 <= 5");
    assert_js(Value::Boolean(false), "5 <= 3");
    assert_js(Value::Boolean(true), "10 > 5");
    assert_js(Value::Boolean(false), "3 > 5");
    assert_js(Value::Boolean(true), "5 >= 5");
    assert_js(Value::Boolean(false), "3 >= 5");
    assert_js(Value::Boolean(true), "5 == 5");
    assert_js(Value::Boolean(false), "5 == 3");
    assert_js(Value::Boolean(true), "5 != 3");
    assert_js(Value::Boolean(false), "5 != 5");
    assert_js(Value::Boolean(true), "1 === 1");
    assert_js(Value::Boolean(false), "1 === true");
    assert_js(Value::Boolean(true), "1 !== true");
    assert_js(Value::Boolean(false), "1 !== 1");
    assert_js(Value::Boolean(true), "true && true");
    assert_js(Value::Boolean(false), "true && false");
    assert_js(Value::Boolean(false), "false && true");
    assert_js(Value::Boolean(true), "true || false");
    assert_js(Value::Boolean(true), "true || true");
    assert_js(Value::Boolean(false), "false || false");
    assert_js(Value::Boolean(false), "!true");
    assert_js(Value::Boolean(true), "!false");
}

#[test]
fn test_statements() {
    assert_js(Value::Number(40), "20;30;40");
    assert_js(Value::Number(91), "34,  48,91");
    assert_js(Value::Number(10), "a = 10");
    assert_js(Value::Number(40), "a = 5,a * 8");
    assert_js(Value::Number(100), "a=10;b = 90;a + b");
    assert_js(Value::Number(40), "a=  b= 20,  a+b");
}
