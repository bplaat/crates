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
    let result = context.eval(script);
    if result.is_err() || result.as_ref().ok() != Some(&expected) {
        let mut context = Context::new();
        context.set_verbose(true);
        let result = context.eval(script).unwrap();
        assert_eq!(expected, result);
    } else {
        assert_eq!(expected, result.ok().unwrap());
    }
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
    assert_js(Value::Number(9), "2 ** 3 + 1");
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
fn test_strings() {
    assert_js(Value::String(String::from("hello")), r#""hello""#);
    assert_js(Value::String(String::from("world")), r#"'world'"#);
    assert_js(
        Value::String(String::from("hello world")),
        r#""hello " + 'world'"#,
    );
    assert_js(
        Value::String(String::from("hello world!")),
        r#""hello " + ('world' + "!")"#,
    );
    assert_js(Value::Boolean(true), r#""hello" == "hello""#);
    assert_js(Value::Boolean(false), r#""hello" == "world""#);
    assert_js(Value::Boolean(false), r#""" != """#);
    assert_js(Value::Boolean(true), r#""test" && true"#);
    assert_js(Value::String(String::from("")), r#""" && true"#);
    assert_js(Value::Boolean(false), r#""" || false"#);
    assert_js(Value::String(String::from("")), r#""" || """#);
    assert_js(Value::Boolean(true), r#"!"""#);
}

#[test]
fn test_assingments() {
    assert_js(Value::Number(10), "a = 10");
    assert_js(Value::Number(15), "a = 5; a += 10");
    assert_js(Value::Number(5), "a = 15; a -= 10");
    assert_js(Value::Number(50), "a = 5; a *= 10");
    assert_js(Value::Number(2), "a = 20; a /= 10");
    assert_js(Value::Number(2), "a = 20; a %= 6");
    assert_js(Value::Number(32), "a = 2; a **= 5");
    assert_js(Value::Number(1), "a = 5; a &= 3");
    assert_js(Value::Number(7), "a = 5; a |= 3");
    assert_js(Value::Number(6), "a = 5; a ^= 3");
    assert_js(Value::Number(16), "a = 8; a <<= 1");
    assert_js(Value::Number(4), "a = 8; a >>= 1");
    assert_js(Value::Number(4), "a = 8; a >>>= 1");
    assert_js(Value::Number(4), "a = 16; a >>= 2");
    assert_js(Value::Number(15), "a = 5; b = 10; a + b");
    assert_js(Value::Number(15), "a = 20; b = 5; a - b");
    assert_js(Value::Number(50), "a = 5; b = 10; a * b");
    assert_js(Value::Number(4), "a = 20; b = 5; a / b");
    assert_js(Value::Number(2), "a = 17; b = 5; a % b");
    assert_js(Value::Number(32), "a = 2; b = 5; a ** b");
    assert_js(Value::Number(1), "a = 5; b = 3; a & b");
    assert_js(Value::Number(7), "a = 5; b = 3; a | b");
    assert_js(Value::Number(6), "a = 5; b = 3; a ^ b");
    assert_js(Value::Number(20), "a = 5; b = 2; a << b");
    assert_js(Value::Number(1), "a = 5; b = 2; a >> b");
    assert_js(Value::Number(1), "a = 5; b = 2; a >>> b");
    assert_js(Value::Number(30), "a = 10; b = 5; c = 15; a + b + c");
    assert_js(Value::Number(100), "a = 10; a += 20; a += 30; a += 40");
    assert_js(Value::Number(24), "a = 6; b = 4; a *= b");
    assert_js(Value::Number(3), "a = 15; b = 5; a /= b");
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
