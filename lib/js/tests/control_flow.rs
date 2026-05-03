/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Statement and control-flow tests.

mod common;

use common::assert_js;
use js::Value;

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
fn test_switch_fall_through() {
    // ES5 12.11 fall-through semantics - test262 S12.11_A1_T1
    assert_js(
        Value::Number(6.0),
        r#"
        function f(v) {
            var r = 0;
            switch (v) {
                case 0: r += 2;
                case 1: r += 4; break;
                case 2: r += 8;
                case 3: r += 16;
                default: r += 32; break;
                case 4: r += 64;
            }
            return r;
        }
        f(0)
        "#,
    );
    assert_js(
        Value::Number(4.0),
        r#"
        function f(v) {
            var r = 0;
            switch (v) {
                case 0: r += 2;
                case 1: r += 4; break;
                case 2: r += 8;
                case 3: r += 16;
                default: r += 32; break;
                case 4: r += 64;
            }
            return r;
        }
        f(1)
        "#,
    );
    assert_js(
        Value::Number(56.0),
        r#"
        function f(v) {
            var r = 0;
            switch (v) {
                case 0: r += 2;
                case 1: r += 4; break;
                case 2: r += 8;
                case 3: r += 16;
                default: r += 32; break;
                case 4: r += 64;
            }
            return r;
        }
        f(2)
        "#,
    );
    assert_js(
        Value::Number(48.0),
        r#"
        function f(v) {
            var r = 0;
            switch (v) {
                case 0: r += 2;
                case 1: r += 4; break;
                case 2: r += 8;
                case 3: r += 16;
                default: r += 32; break;
                case 4: r += 64;
            }
            return r;
        }
        f(3)
        "#,
    );
    assert_js(
        Value::Number(64.0),
        r#"
        function f(v) {
            var r = 0;
            switch (v) {
                case 0: r += 2;
                case 1: r += 4; break;
                case 2: r += 8;
                case 3: r += 16;
                default: r += 32; break;
                case 4: r += 64;
            }
            return r;
        }
        f(4)
        "#,
    );
    // Simple sequential fall-through without break
    assert_js(
        Value::Number(3.0),
        r#"
        var x = 0;
        switch (1) {
            case 1: x += 1;
            case 2: x += 2;
        }
        x
        "#,
    );
}

#[test]
fn test_switch_strict_equality() {
    // ES5 12.11: switch uses strict equality (===), not loose (==)
    // test262 S12.11_A1_T1: SwitchTest(true) === 32 (default), not case 1
    assert_js(
        Value::Number(32.0),
        r#"
        function f(v) {
            var r = 0;
            switch (v) {
                case 0: r += 2;
                case 1: r += 4; break;
                default: r += 32; break;
                case 4: r += 64;
            }
            return r;
        }
        f(true)
        "#,
    );
    // '0' should NOT match case 0 (string vs number)
    assert_js(
        Value::String(String::from("default")),
        r#"switch ('0') { case 0: 'zero'; break; default: 'default'; }"#,
    );
    // 1 should NOT match case '1' (number vs string)
    assert_js(
        Value::String(String::from("default")),
        r#"switch (1) { case '1': 'string-one'; break; default: 'default'; }"#,
    );
    // null should NOT match case 0
    assert_js(
        Value::String(String::from("default")),
        r#"switch (null) { case 0: 'zero'; break; default: 'default'; }"#,
    );
    // false should NOT match case 0
    assert_js(
        Value::String(String::from("default")),
        r#"switch (false) { case 0: 'zero'; break; default: 'default'; }"#,
    );
}

#[test]
fn test_switch_default_in_middle() {
    // Default clause in the middle of cases: if no case matches, start from default
    // and fall through into subsequent cases.
    assert_js(
        Value::Number(99.0),
        r#"
        var x = 0;
        switch (5) {
            case 1: x = 1; break;
            default: x = 99; break;
            case 3: x = 3; break;
        }
        x
        "#,
    );
    // Case after default matches - default should NOT run
    assert_js(
        Value::Number(3.0),
        r#"
        var x = 0;
        switch (3) {
            case 1: x = 1; break;
            default: x = 99; break;
            case 3: x = 3; break;
        }
        x
        "#,
    );
    // Fall-through from default into subsequent case
    assert_js(
        Value::Number(3.0),
        r#"
        var x = 0;
        switch (5) {
            case 1: x = 1; break;
            default: x = 99;
            case 3: x = 3; break;
        }
        x
        "#,
    );
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
        Value::Number(2.0),
        "let i = 0; do { i++; if (i == 2) break; } while (i < 10); i",
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
fn test_exceptions() {
    assert_js(Value::Number(42.0), "try { throw 42; } catch (e) { e }");
    assert_js(
        Value::String(String::from("boom")),
        "try { throw 'boom'; } catch (e) { e }",
    );
    assert_js(
        Value::Number(3.0),
        "let x = 0; try { throw 1; } catch (e) { x = e + 1; } finally { x += 1; } x",
    );
    assert_js(
        Value::Number(2.0),
        "function test() { try { return 1; } finally { return 2; } } test()",
    );
    assert_js(
        Value::Number(5.0),
        "function test() { try { throw 1; } catch (e) { return e + 1; } finally { return 5; } } test()",
    );
}
