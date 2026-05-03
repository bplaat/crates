/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Type coercion tests (Phase 1).
//! Test cases modeled after tc39/test262 test/language/expressions/addition/ and
//! test/language/operators/.

mod common;

use common::assert_js;
use js::Value;

#[test]
fn test_addition_string_coercion() {
    // ES5 11.6.1: if either operand is a string, stringify both and concatenate
    assert_js(Value::String("hello1".into()), r#""hello" + 1"#);
    assert_js(Value::String("1hello".into()), r#"1 + "hello""#);
    assert_js(Value::String("42".into()), r#"4 + "2""#);
    assert_js(Value::String("truehello".into()), r#"true + "hello""#);
    assert_js(Value::String("nullhello".into()), r#"null + "hello""#);
    assert_js(
        Value::String("undefinedhello".into()),
        r#"undefined + "hello""#,
    );
    assert_js(Value::String("12".into()), r#"1 + "2""#);
    assert_js(Value::String("1.52.5".into()), r#"1.5 + "2.5""#);
}

#[test]
fn test_subtract_coercion() {
    // ES5: subtraction always converts to numbers
    assert_js(Value::Number(4.0), r#""5" - 1"#);
    assert_js(Value::Number(3.0), r#"5 - "2""#);
    assert_js(Value::Number(4.0), r#""5" - "1""#);
    assert_js(Value::Number(0.0), r#"true - true"#);
    assert_js(Value::Number(1.0), r#"true - false"#);
    assert_js(Value::Number(0.0), r#"null - 0"#);
}

#[test]
fn test_multiply_coercion() {
    assert_js(Value::Number(6.0), r#""2" * "3""#);
    assert_js(Value::Number(10.0), r#"true * 10"#);
    assert_js(Value::Number(0.0), r#"null * 5"#);
    assert_js(Value::Boolean(true), r#"isNaN(undefined * 1)"#);
}

#[test]
fn test_divide_coercion() {
    assert_js(Value::Number(2.5), r#""5" / "2""#);
    assert_js(Value::Number(0.5), r#"true / 2"#);
}

#[test]
fn test_comparison_strings() {
    // Lexicographic string comparison
    assert_js(Value::Boolean(true), r#""a" < "b""#);
    assert_js(Value::Boolean(false), r#""b" < "a""#);
    assert_js(Value::Boolean(true), r#""apple" < "banana""#);
    assert_js(Value::Boolean(true), r#""abc" <= "abc""#);
    assert_js(Value::Boolean(true), r#""z" > "a""#);
    assert_js(Value::Boolean(true), r#""b" >= "a""#);
    // Numbers coerce when not both strings
    assert_js(Value::Boolean(true), r#""10" > 9"#);
}

#[test]
fn test_unary_minus_coercion() {
    assert_js(Value::Number(-5.0), r#"-"5""#);
    assert_js(Value::Number(-1.0), r#"-true"#);
    assert_js(Value::Number(0.0), r#"-null"#);
    assert_js(Value::Boolean(true), r#"isNaN(-undefined)"#);
    assert_js(Value::Boolean(true), r#"isNaN(-"abc")"#);
}

#[test]
fn test_add_assign_string_coercion() {
    // += with string operands should concatenate
    assert_js(
        Value::String("hello world".into()),
        r#"let s = "hello"; s += " world"; s"#,
    );
    assert_js(
        Value::String("count1".into()),
        r#"let s = "count"; s += 1; s"#,
    );
}

#[test]
fn test_number_to_string_conversion() {
    // JS-specific number formatting
    assert_js(Value::String("Infinity".into()), r#"String(Infinity)"#);
    assert_js(Value::String("-Infinity".into()), r#"String(-Infinity)"#);
    assert_js(Value::String("NaN".into()), r#"String(NaN)"#);
    assert_js(Value::String("0".into()), r#"String(0)"#);
    assert_js(Value::String("42".into()), r#"String(42)"#);
    assert_js(Value::String("3.14".into()), r#"String(3.14)"#);
}

#[test]
fn test_bitwise_coercion() {
    // Bitwise ops always convert to i32
    assert_js(Value::Number(5.0), r#""5" & 7"#);
    assert_js(Value::Number(7.0), r#""5" | 2"#);
    assert_js(Value::Number(0.0), r#"null & 5"#);
}
