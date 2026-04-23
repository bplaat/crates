/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Assignment and expression operator tests.

mod common;

use common::assert_js;
use js::Value;

#[test]
fn test_assignments() {
    assert_js(Value::Number(10.0), "a = 10");
    assert_js(Value::Number(15.0), "a = 5; a += 10");
    assert_js(Value::Number(5.0), "a = 15; a -= 10");
    assert_js(Value::Number(50.0), "a = 5\n a *= 10");
    assert_js(Value::Number(2.0), "a = 20\n a /= 10");
    assert_js(Value::Number(2.0), "a = 20; a %= 6");
    assert_js(Value::Number(32.0), "a = 2\n a **= 5");
    assert_js(Value::Number(1.0), "a = 5; a &= 3");
    assert_js(Value::Number(7.0), "a = 5; a |= 3");
    assert_js(Value::Number(6.0), "a = 5; a ^= 3");
    assert_js(Value::Number(16.0), "a = 8; a <<= 1");
    assert_js(Value::Number(4.0), "a = 8; a >>= 1");
    assert_js(Value::Number(4.0), "a = 8; a >>>= 1");
    assert_js(Value::Number(4.0), "a = 16; a >>= 2");
    assert_js(Value::Number(15.0), "a = 5; b = 10; a + b");
    assert_js(Value::Number(15.0), "a = 20; b = 5; a - b");
    assert_js(Value::Number(50.0), "a = 5; b = 10; a * b");
    assert_js(Value::Number(4.0), "a = 20; b = 5; a / b");
    assert_js(Value::Number(2.0), "a = 17; b = 5; a % b");
    assert_js(Value::Number(32.0), "a = 2;\n b = 5;\n a ** b");
    assert_js(Value::Number(1.0), "a = 5; b = 3; a & b");
    assert_js(Value::Number(7.0), "a = 5; b = 3; a | b");
    assert_js(Value::Number(6.0), "a = 5; b = 3 \n a ^ b");
    assert_js(Value::Number(20.0), "a = 5; b = 2; a << b");
    assert_js(Value::Number(1.0), "a = 5; b = 2; a >> b");
    assert_js(Value::Number(1.0), "a = 5; b = 2; a >>> b");
    assert_js(Value::Number(30.0), "a = 10; b = 5; c = 15; a + b + c");
    assert_js(Value::Number(100.0), "a = 10 \n a += 20; a += 30; a += 40");
    assert_js(Value::Number(24.0), "a = 6;\n b = 4; a *= b");
    assert_js(Value::Number(3.0), "a = 15; b = 5; a /= b");
    assert_js(Value::Number(5.0), "a = 0; a ||= 5");
    assert_js(Value::Number(10.0), "a = 10; a ||= 5");
    assert_js(Value::Number(0.0), "a = 5; a &&= 0");
    assert_js(Value::Number(10.0), "a = 5; a &&= 10");
    assert_js(Value::Number(8.0), "a = 0; b = 8; a ||= b");
    assert_js(Value::Number(5.0), "a = 3; b = 5; a &&= b");
}

#[test]
fn test_comma_operator() {
    assert_js(Value::Number(20.0), " (10, 20) ");
    assert_js(Value::Number(30.0), " (5 + \n5, 15 + 15) ");
    assert_js(Value::Number(3.0), " (1, 2, \n3) ");
    assert_js(Value::Number(100.0), " (50 * \n2, 25 * 4, 100) ");
    assert_js(Value::Number(15.0), " (a = 10, b = 5, a + b) ");
    assert_js(Value::Number(50.0), " (a = 20\n, b = 30, a + b) ");
    assert_js(Value::Number(7.0), " (x = 3, y = 4, x + y) ");
    assert_js(Value::Number(0.0), " (x = 0\n,\n y = 1, x) ");
    assert_js(Value::Number(1.0), " (x = 0, y = 1, y) ");
}

#[test]
fn test_ternary() {
    assert_js(Value::Number(10.0), "true ? 10 : 20");
    assert_js(Value::Number(20.0), "false ? 10 : 20");
    assert_js(Value::String(String::from("yes")), "true ? 'yes' : 'no'");
    assert_js(Value::String(String::from("no")), "false ? 'yes' : 'no'");
    assert_js(Value::Number(20.0), "false ? 10 : (true ? 20 : 30)");
    assert_js(Value::Number(20.0), "true ? (false ? 10 : 20) : 30");
    assert_js(Value::Number(15.0), "true ? (true ? 15 : 10) : 20");
    assert_js(Value::Number(10.0), "true ? (false ? 15 : 10) : 20");
    assert_js(Value::Number(20.0), "false ? (true ? 15 : 10) : 20");
    assert_js(Value::Number(25.0), "true ? (5 > 3 ? 25 : 5) : 10");
    assert_js(
        Value::Number(25.0),
        "true ? (5 > 3 \n? 25 : 5) : (5 < 3 ? 15 : 10)",
    );
    assert_js(Value::Number(10.0), "5 > 10 ? 20 : (3 < 5 ? 10 : 15)");
    assert_js(
        Value::Number(50.0),
        "true ? (true ? (true ? 50 : \n40) : 30) : 20",
    );
    assert_js(
        Value::Boolean(true),
        "true ? (5 > 3 \n? true : false) : false",
    );
    assert_js(
        Value::Number(7.0),
        "5 > 3 ? (2 < 8 ? 7 : 6) : (1 > 0 ? 5 : 4)",
    );
}

#[test]
fn test_increment_decrement() {
    assert_js(Value::Number(1.0), "let a = 0; ++a");
    assert_js(Value::Number(0.0), "let a = 0; a++");
    assert_js(Value::Number(1.0), "let a = 1; a");
    assert_js(Value::Number(-1.0), "let a = 0; --a");
    assert_js(Value::Number(0.0), "let a = 0; a--");
    assert_js(Value::Number(-1.0), "let a = -1; a");
    assert_js(Value::Number(5.0), "let a = 4; ++a; a");
    assert_js(Value::Number(5.0), "let a = 4; a++; a");
    assert_js(Value::Number(3.0), "let a = 4; --a; a");
    assert_js(Value::Number(3.0), "let a = 4; a--; a");
    assert_js(Value::Number(11.0), "let a = 10; let b = ++a; b");
    assert_js(Value::Number(10.0), "let a = 10; let b = a++; b");
    assert_js(Value::Number(9.0), "let a = 10; let b = --a; b");
    assert_js(Value::Number(10.0), "let a = 10; let b = a--; b");
}
