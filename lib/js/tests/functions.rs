/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Function and scope tests.

mod common;

use common::assert_js;
use js::Value;

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
    assert_js(Value::Number(5.0), "for (var i = 0; i < 5; i++) { } i");
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
