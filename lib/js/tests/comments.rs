/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Comment parsing tests.

mod common;

use common::assert_js;
use js::Value;

#[test]
fn test_comments() {
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
