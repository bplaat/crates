/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Primitive value and operator tests.

mod common;

use common::assert_js;
use js::Value;

#[test]
fn test_keywords() {
    assert_js(Value::Undefined, "undefined");
    assert_js(Value::Null, "null");
    assert_js(Value::Boolean(true), "true");
    assert_js(Value::Boolean(false), "false");
}

#[test]
fn test_number_literals() {
    assert_js(Value::Number(0.0), "0");
    assert_js(Value::Number(42.0), "42");
    assert_js(Value::Number(-7.0), "-7");
    assert_js(Value::Number(1234567890.0), "1234567890");
    assert_js(Value::Number(255.0), "0xFF");
    assert_js(Value::Number(256.0), "0x100");
    assert_js(Value::Number(8.0), "0o10");
    assert_js(Value::Number(64.0), "0o100");
    assert_js(Value::Number(5.0), "0b101");
    assert_js(Value::Number(16.0), "0b10000");
    assert_js(Value::Number(1000.0), "1e3");
    assert_js(Value::Number(0.001), "1e-3");
    assert_js(Value::Number(12340.0), "1.234e4");
    assert_js(Value::Number(0.001234), "1.234e-3");
}

#[test]
fn test_int_arithmetic() {
    assert_js(Value::Number(0.0), "0");
    assert_js(Value::Number(42.0), "42");
    assert_js(Value::Number(21.0), "5+20-4");
    assert_js(Value::Number(41.0), " 12 + 34 - 5 ");
    assert_js(Value::Number(47.0), "5+6*7");
    assert_js(Value::Number(15.0), "5*(9-6)");
    assert_js(Value::Number(4.0), "(3 + 5) / 2");
    assert_js(Value::Number(16.0), "2 ** 4");
    assert_js(Value::Number(9.0), "2 ** 3 + 1");
    assert_js(Value::Number(2.0), "20 % 3");
    assert_js(Value::Number(34.0), "- -34");
    assert_js(Value::Number(-40.0), "- - -40");
    assert_js(Value::Number(1.0), "5 & 3");
    assert_js(Value::Number(7.0), "5 | 3");
    assert_js(Value::Number(6.0), "5 ^ 3");
    assert_js(Value::Number(16.0), "8 << 1");
    assert_js(Value::Number(4.0), "8 >> 1");
    assert_js(Value::Number(0.0), "8 >> 4");
    assert_js(Value::Number(4.0), "8 >>> 1");
    assert_js(Value::Number(0.0), "8 >>> 4");
}

#[test]
fn test_float_arithmetic() {
    assert_js(Value::Number(3.19), "3.19");
    assert_js(Value::Number(2.5), "5.0 / 2.0");
    assert_js(Value::Number(7.5), "2.5 + 5.0");
    assert_js(Value::Number(2.5), "7.5 - 5.0");
    assert_js(Value::Number(15.5), "3.1 * 5.0");
    assert_js(Value::Number(2.0), "10.0 / 5.0");
    assert_js(Value::Number(1.5), "6.5 % 2.5");
    assert_js(Value::Number(8.0), "2.0 ** 3.0");
    assert_js(Value::Number(0.125), "1.0 / 8.0");
    assert_js(Value::Number(12.34), "10.0 + 2.34");
    assert_js(Value::Number(0.1), "0.5 / 5.0");
    assert_js(Value::Number(-3.5), "-3.5");
    assert_js(Value::Number(5.5), "2.75 * 2.0");
    assert_js(Value::Number(99.99), "99.99");
    assert_js(Value::Number(0.001), "0.001");
    assert_js(Value::Number(1000.5), "500.25 + 500.25");
    assert_js(Value::Number(1.0), "1.5 - 0.5");
    assert_js(Value::Number(10.0), "(2.5 + 2.5) * 2.0");
    assert_js(Value::Number(3.0), "7.5 / 2.5");
    assert_js(Value::Number(1.0), "5.7 & 3.2");
    assert_js(Value::Number(7.0), "5.7 | 3.2");
    assert_js(Value::Number(6.0), "5.7 ^ 3.2");
    assert_js(Value::Number(16.0), "8.9 << 1.1");
    assert_js(Value::Number(4.0), "8.9 >> 1.5");
    assert_js(Value::Number(4.0), "8.9 >>> 1.5");
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
        Value::String(String::from("hello\nworld")),
        r#""hello\nworld""#,
    );
    assert_js(
        Value::String(String::from("hello\tworld")),
        r#""hello\tworld""#,
    );
    assert_js(
        Value::String(String::from("hello\\world")),
        r#""hello\\world""#,
    );
    assert_js(
        Value::String(String::from("hello\"world")),
        r#""hello\"world""#,
    );
    assert_js(
        Value::String(String::from("hello'world")),
        r#"'hello\'world'"#,
    );
    assert_js(
        Value::String(String::from("hello\rworld")),
        r#""hello\rworld""#,
    );
    assert_js(
        Value::String(String::from_utf8(vec![8, b'w', b'o', b'r', b'l', b'd']).unwrap()),
        r#""\bworld""#,
    );
    assert_js(
        Value::String(String::from_utf8(vec![12, b'w', b'o', b'r', b'l', b'd']).unwrap()),
        r#""\fworld""#,
    );
    assert_js(
        Value::String(String::from_utf8(vec![11, b'w', b'o', b'r', b'l', b'd']).unwrap()),
        r#""\vworld""#,
    );
    assert_js(
        Value::String(String::from("hello\0world")),
        r#""hello\0world""#,
    );
    assert_js(
        Value::String(String::from("helloAworld")),
        r#""hello\x41world""#,
    );
    assert_js(
        Value::String(String::from("helloAworld")),
        r#""hello\u0041world""#,
    );
    assert_js(
        Value::String(String::from("line1\nline2\ttab")),
        r#""line1\nline2\ttab""#,
    );
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
fn test_typeof() {
    assert_js(Value::String(String::from("undefined")), "typeof undefined");
    assert_js(Value::String(String::from("object")), "typeof null");
    assert_js(Value::String(String::from("boolean")), "typeof true");
    assert_js(Value::String(String::from("boolean")), "typeof false");
    assert_js(Value::String(String::from("number")), "typeof 42");
    assert_js(Value::String(String::from("string")), r#"typeof "hello""#);
    assert_js(Value::String(String::from("string")), "typeof 'world'");
}

#[test]
fn test_division_by_zero() {
    // ES5 11.5.2 (IEEE 754) - test262 S11.5.2_A4 series
    assert_js(Value::Number(f64::INFINITY), "5 / 0");
    assert_js(Value::Number(f64::NEG_INFINITY), "-5 / 0");
    assert_js(Value::Boolean(true), "isNaN(0 / 0)");
    assert_js(Value::Number(0.0), "1 / Infinity");
    assert_js(Value::Number(0.0), "-1 / -Infinity");
    assert_js(Value::Number(f64::INFINITY), "1 / 0 + 1 / 0");
    assert_js(Value::Number(f64::INFINITY), "let a = 5; a /= 0; a");
}

#[test]
fn test_number_infinity_arithmetic() {
    // ES5 11.5 arithmetic with Infinity
    assert_js(Value::Number(f64::INFINITY), "Infinity + 1");
    assert_js(Value::Number(f64::INFINITY), "Infinity + Infinity");
    assert_js(Value::Boolean(true), "isNaN(Infinity - Infinity)");
    assert_js(Value::Number(f64::INFINITY), "Infinity * 2");
    assert_js(Value::Boolean(true), "isNaN(Infinity * 0)");
    assert_js(Value::Number(f64::INFINITY), "Infinity / 2");
    assert_js(Value::Number(0.0), "2 / Infinity");
    assert_js(Value::Boolean(true), "isNaN(Infinity / Infinity)");
    assert_js(Value::Number(f64::NEG_INFINITY), "-Infinity");
    assert_js(Value::Number(f64::NEG_INFINITY), "Infinity * -1");
}
