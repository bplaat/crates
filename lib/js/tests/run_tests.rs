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
fn test_statements() {
    assert_js(Value::Number(40.0), "20;30;40");
    assert_js(Value::Number(91.0), "34,  48,91");
    assert_js(Value::Number(10.0), "a = 10");
    assert_js(Value::Number(40.0), "a = \n5,a * 8");
    assert_js(Value::Number(100.0), "a=10;b = 90;a + b");
    assert_js(Value::Number(40.0), "a= \n b= 20,\n  a+b");
    assert_js(
        Value::String(String::from("Hello World")),
        "a='Hello';a+=' World';a",
    );
}

#[test]
fn test_var_let_const() {
    assert_js(Value::Number(10.0), "var a = 10; a");
    assert_js(Value::Number(20.0), "let a = 20 \n a");
    assert_js(Value::Number(30.0), "const a = \n30; a");
    assert_js(Value::Number(25.0), "var \n a \n = 10 \n let b = 15; a + b");
    assert_js(Value::Number(50.0), "let a = 20; const b \n= 30; a + b");
    assert_js(
        Value::String(String::from("Hello JS")),
        "const greeting = 'Hello'; let target = ' JS'; greeting + target",
    );
}

#[test]
fn test_if() {
    assert_js(Value::Number(10.0), "if \n(true) { 10 } else { 20 }");
    assert_js(Value::Number(20.0), "if (false) \n{ 10 }\n else \n{ 20 }");
    assert_js(
        Value::Number(15.0),
        "let a = 15; if (a > 10) { a } else { 10 }",
    );
    assert_js(
        Value::Number(8.0),
        "let a = 8; if (a > 10) { 10 } else { a }",
    );
    assert_js(
        Value::Number(30.0),
        "let a = 30; if (a < 20) { 20 } else if (a < 40) { a } else { 50 }",
    );
    assert_js(
        Value::Number(50.0),
        "let a = 50; if (a < 20) { 20 } else if (a < 40) { 30 } else { a }",
    );
    assert_js(
        Value::Number(15.0),
        "let a = 15; if (a > 10) { if (a < 20) { a } else { 20 } } else { 10 }",
    );
    assert_js(
        Value::Number(25.0),
        "let a = 25; if (a > 10) { if (a > 20) { a } else { 20 } } else { 10 }",
    );
    assert_js(
        Value::Number(10.0),
        "let a = 5; if (a > 10) { if (a > 20) { a } else { 20 } } else { if (a > 0) { 10 } else { 5 } }",
    );
}

#[test]
fn test_switch() {
    assert_js(
        Value::Number(20.0),
        "let a = 2; switch (a) { case 1: 10; break; case 2: 20; break; default: 30; }",
    );
    assert_js(
        Value::Number(10.0),
        "let a = 1; \n switch ( \n a \n) \n { \n case 1: 10; break; case 2: 20; break; default: 30; }",
    );
    assert_js(
        Value::Number(30.0),
        "let a = 5; switch (a) \n { case 1: 10; break; case 2: 20; break; default: 30; }",
    );
    assert_js(
        Value::Number(30.0),
        "let a = 3; switch (a) { case 1: 10; break; case 2: 20; break; default: 30; }",
    );
    assert_js(
        Value::Number(60.0),
        "let a = 2; switch (a) { case 1: 10; break; case 2: 30 + 30; break; default: 50; }",
    );
    assert_js(
        Value::Number(25.0),
        "let a = 1; switch (a) { case 1: 5 * 5; break; case 2: 10 + 10; break; default: 30; }",
    );
    assert_js(
        Value::Number(15.0),
        "let a = 1; let b = 5; switch (a) { case 1: \n b + 10; \n break; case 2: b * 2; break; default: 0; }",
    );
    assert_js(
        Value::Number(100.0),
        "let x = 10; switch (x) { case 10: \n 100; \n break; \n case \n 20: 200; break; case 30: 300; break; default: 0; }",
    );
    assert_js(
        Value::Number(200.0),
        "let x = 20; switch (x) { case 10: 100; \n break; case 20: 200; break; case 30: 300; break; default: 0; }",
    );
    assert_js(
        Value::Number(0.0),
        "let x = 40; switch (x) { \n case 10: 100; break; case 20: 200; break; case 30: 300; break; default: 0; }",
    );
    assert_js(
        Value::Number(45.0),
        "let a = 2; let b = 15; switch (a) { case 1: b + 5; break; case 2: b + 30; break; case 3: b * 3; break; default: 0; }",
    );
    assert_js(
        Value::Number(55.0),
        "let n = 3; switch (n) { case 1: 10; break; case 2: 25; break; case 3: n * 18 + 1; break; default: 0; }",
    );
    assert_js(
        Value::String(String::from("two")),
        r#"let a = 2; switch (a) { case 1: "one"; break; case 2: "two"; break; default: "other"; }"#,
    );
    assert_js(
        Value::String(String::from("one")),
        r#"let a = 1; switch (a) { case 1: "one"; break; case 2: "two"; break; case 3: "three"; break; default: "other"; }"#,
    );
    assert_js(
        Value::Boolean(true),
        "let a = 1; switch (a) { case 1: true; break; break; case 2: false; break; default: null; }",
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

#[test]
fn test_loops() {
    assert_js(Value::Number(4.0), "let i = 0; while (i < 5) { i++; }");
    assert_js(Value::Number(5.0), "let i = 0; while (i < 5) { i++; } i");
    assert_js(
        Value::Number(3.0),
        "let i = 0; while (i < 10) { if (i == 3) break; i++; } i",
    );
    assert_js(
        Value::Number(15.0),
        "let i = 0; let sum = 0; while (i < 5) { i++; sum += i; }",
    );
    assert_js(
        Value::Number(2.0),
        "let i = 0; while (i < 10) { i++; if (i == 2) break; } i",
    );
    assert_js(
        Value::Number(13.0),
        "let i = 0; let sum = 0; while (i < 5) { i++; if (i == 2) continue; sum += i; }",
    );
    assert_js(
        Value::Number(12.0),
        "let i = 0; let sum = 0; while (i < 5) { i++; if (i == 3) continue; sum += i; } sum",
    );
    assert_js(Value::Number(4.0), "let i = 0; do { i++; } while (i < 5)");
    assert_js(
        Value::Number(5.0),
        "let i = 0; do { i++; } while (i < 5); i",
    );
    assert_js(
        Value::Number(3.0),
        "let i = 0; do { if (i == 3) break; i++; } while (i < 10); i",
    );
    assert_js(
        Value::Number(15.0),
        "let i = 0; let sum = 0; do { i++; sum += i; } while (i < 5); sum",
    );
    assert_js(
        Value::Number(1.0),
        "let i = 0; do { i++; if (i == 2) break } while (i < 10)",
    );
    assert_js(
        Value::Number(13.0),
        "let i = 0; let sum = 0; do { i++; if (i == 2) continue; sum += i; } while (i < 5); sum",
    );
    assert_js(
        Value::Number(12.0),
        "let i = 0; let sum = 0; do { i++; if (i == 3) continue; sum += i; } while (i < 5);",
    );
    assert_js(
        Value::Number(20.0),
        "let i = 0; let sum = 0; while (i < 4) { let j = 0; while (j < 5) { sum++; j++; } i++; } sum",
    );
    assert_js(
        Value::Number(12.0),
        "let i = 0; let sum = 0; while (i < 3) { let j = 0; while (j < 4) { sum += 1; j++; } i++; } sum",
    );
    assert_js(
        Value::Number(10.0),
        "let sum = 0; for (let i = 0; i < 5; i++) { sum += i; } sum",
    );
    assert_js(
        Value::Number(5.0),
        "let i = 0; for (i = 0; i < 5; i++) { } i",
    );
    assert_js(
        Value::Number(3.0),
        "let i = 0; for (i = 0; i < 10; i++) { if (i == 3) break; } i",
    );
    assert_js(
        Value::Number(8.0),
        "let sum = 0; for (let i = 0; i < 5; i++) { if (i == 2) continue; sum += i; }",
    );
    assert_js(
        Value::Number(8.0),
        "let sum = 0; for (let i = 0; i < 5; i++) { if (i == 2) continue; sum += i; } sum",
    );
    assert_js(
        Value::Number(15.0),
        "let sum = 0; for (let i = 1; i <= 5; i++) { sum += i; } sum",
    );
    assert_js(
        Value::Number(20.0),
        "let sum = 0; for (let i = 0; i < 4; i++) { for (let j = 0; j < 5; j++) { sum++; } } sum",
    );
    assert_js(
        Value::Number(12.0),
        "let sum = 0; for (let i = 0; i < 3; i++) { for (let j = 0; j < 4; j++) { sum += 1; } } sum",
    );
    assert_js(
        Value::Number(6.0),
        "let sum = 0; for (let i = 0; i < 3; i++) { for (let j = 0; j < 2; j++) { sum++; } } sum",
    );
    assert_js(
        Value::Number(9.0),
        "let sum = 0; for (let i = 0; i < 3; i++) { for (let j = 0; j < 4; j++) { if (j == 2) continue; sum++; } } sum",
    );
    assert_js(
        Value::Number(3.0),
        "let sum = 0; for (let i = 0; i < 3; i++) { for (let j = 0; j < 4; j++) { if (j == 1) break; sum++; } } sum",
    );
    assert_js(
        Value::Number(6.0),
        "let sum = 0; for (let i = 0; i < 3; i++) { if (i == 2) break; for (let j = 0; j < 3; j++) { sum++; } } sum",
    );
}

#[test]
fn test_loop_labels() {
    assert_js(Value::Number(3.0), "x: { 3; break x; 6 }");
    assert_js(Value::Number(3.0), "x\n:\n{ 3; break x; 6 }");
    assert_js(Value::Number(7.0), "x :  if(true){7;break x; 6}");
    assert_js(
        Value::Number(3.0),
        "let i = 0; outer: while (i < 10) \n { let j = 0; while (j < 10) { if (i == 3) break outer; j++; } i++; } i",
    );
    assert_js(
        Value::Number(1.0),
        "let i = 0; outer: while (i < 5) { let j = 0; \n while (j < 5) { if (j == 2) { i++; break outer; } j++; } i++; } i",
    );
    assert_js(
        Value::Number(4.0),
        "let sum = 0; outer: for (let i = 0; i < 5; i++) { for (let j = 0; j < 5; j++) { if (i + j == 4) break outer; sum++; } } sum",
    );
    assert_js(
        Value::Number(10.0),
        "let sum = 0; outer: for (let i = 0; \n i < 5; \n i++) { for (let j = 0; j < 5; j++) { if (j == 2) continue outer; sum++; } } sum",
    );
}

#[test]
fn test_function() {
    assert_js(
        Value::String(String::from("function")),
        "function add(a, b) { return a + b; } typeof add",
    );
    assert_js(
        Value::Number(25.0),
        "function add(a, b) { return (4, a + b); } add(10, 15);",
    );
    assert_js(
        Value::Number(25.0),
        "function add(a, b) { return a + b; } add(10, 15);",
    );
    assert_js(
        Value::Undefined,
        "function add(a, b) { return;a + b; } add(10, 15);",
    );
    assert_js(
        Value::Undefined,
        "function add(a, b) { return\na + b; } add(10, 15);",
    );
    assert_js(
        Value::Number(120.0),
        "function multiply(x, y) { return x * y; } multiply(10, 12);",
    );
    assert_js(
        Value::Number(49.0),
        "function square(n) { return n * n; } square(7);",
    );
    assert_js(
        Value::Number(6.0),
        "function factorial(n) { if (n <= 1) { return 1; } else { return n * factorial(n - 1); } } factorial(3);",
    );
    assert_js(
        Value::Number(120.0),
        "function factorial(n) { if (n <= 1) { return 1; } else { return n * factorial(n - 1); } } factorial(5);",
    );
    assert_js(
        Value::Number(55.0),
        "function fibonacci(n) { if (n <= 1) { return n; } else { return fibonacci(n - 1) + fibonacci(n - 2); } } fibonacci(10);",
    );

    assert_js(Value::Number(49.0), "const square = n => n * n; square(7);");
    assert_js(
        Value::Number(64.0),
        "const square = (n) => (n) * n; square(8);",
    );
    assert_js(
        Value::Number(11.0),
        "const fmul = (a, b, c) => { let x = a * b; x += c; return x; }; fmul(2, 3, 5);",
    );
    assert_js(
        Value::Number(100.0),
        "const multiply = (x, y) => x * y; multiply(10, 10);",
    );
    assert_js(
        Value::Number(15.0),
        "const sum = (a, b, c) => a + b + c; sum(3, 5, 7);",
    );
    assert_js(Value::Number(5.0), "const identity = x => x; identity(5);");
}

#[test]
fn test_scoping() {
    assert_js(
        Value::Number(10.0),
        "var a = 10; function test() { return a; } test();",
    );
    assert_js(
        Value::Number(20.0),
        "let a = 10; function test() { let a = 20; return a; } test();",
    );
    assert_js(
        Value::Number(10.0),
        "let a = 10; function test() { let a = 20; return a; } test(); a",
    );
    assert_js(
        Value::Number(30.0),
        "let a = 10; function test() { a = 30; return a; } test(); a",
    );
    assert_js(
        Value::Number(15.0),
        "const a = 15; function test() { return a; } test();",
    );
    assert_js(
        Value::Number(15.0),
        "function sum() { let sum = 0; let i = 0; while (arguments[i] !== undefined) { sum += arguments[i]; i++; } return sum; } sum(5, 5, 5);",
    );
    assert_js(
        Value::Number(5.0),
        "function outer() { let x = 5; return x; } outer();",
    );
    assert_js(
        Value::Number(15.0),
        "function outer() { let x = 5; function inner() { return x + 10; } return inner(); } outer();",
    );
    assert_js(
        Value::Number(25.0),
        "function outer() { let x = 5; function inner() { let x = 20; return x; } return inner() + x; } outer();",
    );
    assert_js(Value::Number(10.0), "let x = 10; { let x = 20; } x");
    assert_js(Value::Number(20.0), "let x = 10; { x = 20; } x");
    assert_js(Value::Number(5.0), "for (let i = 0; i < 5; i++) { } i");
    assert_js(
        Value::Number(30.0),
        "let sum = 0; for (let i = 0; i < 5; i++) { let x = 6; sum += x; } sum",
    );
    assert_js(
        Value::Number(10.0),
        "let a = 10; while (true) { let a = 20; break; } a",
    );
    assert_js(
        Value::Number(50.0),
        "let x = 50; if (true) \n{ \n let x = 100; } x",
    );
    assert_js(Value::Number(100.0), "let x = 50; if (true) { x = 100; } x");
    assert_js(
        Value::Number(10.0),
        "var a = 10; function test() { return a; } test();",
    );
    assert_js(
        Value::Number(20.0),
        "var a = 10; function test() { var a = 20; return a; } test();",
    );
    assert_js(
        Value::Number(30.0),
        "var a = 10; function test() { a = 30; } test(); a",
    );
    assert_js(
        Value::Number(30.0),
        "function outer() { let x = 10; function middle() { let y = 20; function inner() { return x + y; } return inner(); } return middle(); } outer();",
    );
    assert_js(
        Value::Number(25.0),
        "const x = 15; function test() { const x = 25; return x; } test();",
    );
    assert_js(
        Value::Number(25.0),
        "let result = 0; for (let i = 0; i < 5; i++) { for (let j = 0; j < 5; j++) { result += 1; } } result",
    );
    assert_js(
        Value::Number(10.0),
        "let sum = 0; for (let i = 0; i < 5; i++) { let x = i; sum += x; } sum",
    );
    assert_js(
        Value::Number(20.0),
        "var a = 10; function test() { var a = 20; { let a = 30; } return a } test()",
    );
    assert_js(
        Value::Number(10.0),
        "var a = 10; function test() { var a = 20; { let a = 30; } return a } test(); a",
    );
}

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
fn test_comments() {
    // Single-line comments
    assert_js(Value::Number(42.0), "// This is a comment\n42");
    assert_js(Value::Number(10.0), "10 // comment at end");
    assert_js(Value::Number(5.0), "// comment\n5");
    assert_js(Value::Number(0.0), "// comment\n// another\n0");
    assert_js(Value::Boolean(true), "true // inline comment");
    assert_js(
        Value::String(String::from("test")),
        r#""test" // string with comment"#,
    );
    assert_js(Value::Number(3.0), "1 + 2 // addition with comment");
    assert_js(Value::Number(8.0), "// start\n4 * 2 // multiply");
    assert_js(Value::Number(15.0), "let a = 10; // declare\n a + 5 // add");
    assert_js(Value::Number(20.0), "/* multi */ 20");
    assert_js(Value::Number(7.0), "/* comment */ 7");
    assert_js(Value::Number(100.0), "/* \n multi line \n */ 100");
    assert_js(Value::Number(50.0), "/* comment */ 50 /* another */");
    assert_js(Value::Boolean(false), "/* false */ false");
    assert_js(
        Value::String(String::from("hello")),
        r#"/* comment */ "hello""#,
    );
    assert_js(Value::Number(25.0), "/* \n block \n */ 5 * 5");
    assert_js(Value::Number(12.0), "let x = 12; /* assign */ x");
    assert_js(Value::Number(6.0), "/* start */ 2 + 4 /* end */");
    assert_js(Value::Number(9.0), "/* multi\nline\ncomment */ 9");
}

#[test]
fn test_arrays() {
    assert_js(Value::Number(3.0), "[1, 2, 3][2]");
    assert_js(Value::Undefined, "[1, 2, 3][5]");
    assert_js(Value::Number(5.0), "let arr = [5, 10, 15]; arr[0]");
    assert_js(Value::Number(15.0), "let arr = [5, 10, 15]; arr[2]");
    assert_js(Value::Undefined, "let arr = [1, 2, 3]; arr[5]");
    assert_js(
        Value::Number(20.0),
        "let arr = [10, 20, 30]; let index = 1; arr[index]",
    );
    assert_js(
        Value::Number(30.0),
        "let arr = [10, 20, 30]; let index = 2; arr[index]",
    );
    assert_js(
        Value::Number(60.0),
        "let arr = [10, 20, 30]; arr[0] + arr[1] + arr[2]",
    );
    assert_js(Value::Number(100.0), "let arr = [25, 50, 75]; arr[0] * 4");
    assert_js(Value::Boolean(false), " [25, 50, 75] == [25, 50, 75] ");
    assert_js(Value::Boolean(true), " let a = [25, 50, 75]; a == a ");

    assert_js(
        Value::Number(15.0),
        "let arr = [1, 2, 3, 4, 5]; let sum = 0; for (let i = 0; i < 5; i++) { sum += arr[i]; } sum",
    );
    assert_js(
        Value::Number(30.0),
        "let arr = [10, 20, 30]; let max = arr[0]; for (let i = 1; i < 3; i++) { if (arr[i] > max) max = arr[i]; } max",
    );
    assert_js(
        Value::Number(24.0),
        "let arr = [1,2,3,4]; let product = 1; let i = 0; while (i < 4) { product *= arr[i]; i++; } product",
    );
    assert_js(
        Value::Number(5.0),
        "function findLength(a) { var length2 = 0; while (a[length2] !== undefined) { length2++; } return length2; } let arr = [1,2,3,4,5]; findLength(arr)",
    );
}
