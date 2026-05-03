/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! ES5 compliance tests covering features from the ECMAScript 5.1 spec.
//! Test cases inspired by test262.

mod common;

use common::assert_js;
use js::Value;

#[test]
fn test_var_declaration() {
    // var without initializer is undefined
    assert_js(Value::Undefined, "var x; x");

    // var inside block leaks to function scope
    assert_js(Value::Number(1.0), "function f() { if (true) { var x = 1; } return x; } f()");

    // Multiple var declarations in one statement
    assert_js(Value::Number(3.0), "var a = 1, b = 2; a + b");

    // var in for loop leaks to enclosing scope
    assert_js(Value::Number(5.0), "for (var i = 0; i < 5; i++) {} i");
}

#[test]
fn test_typeof_operator() {
    assert_js(
        Value::String(String::from("undefined")),
        "typeof undefined",
    );
    assert_js(Value::String(String::from("boolean")), "typeof true");
    assert_js(Value::String(String::from("number")), "typeof 42");
    assert_js(Value::String(String::from("string")), "typeof 'hello'");
    assert_js(
        Value::String(String::from("function")),
        "typeof function(){}",
    );
    assert_js(Value::String(String::from("object")), "typeof null");
    assert_js(Value::String(String::from("object")), "typeof {}");
    // typeof undeclared variable returns 'undefined' without throwing
    assert_js(
        Value::String(String::from("undefined")),
        "typeof notDeclaredVar",
    );
}

#[test]
fn test_equality() {
    // Strict equality
    assert_js(Value::Boolean(true), "1 === 1");
    assert_js(Value::Boolean(false), "1 === '1'");
    assert_js(Value::Boolean(false), "null === undefined");
    assert_js(Value::Boolean(false), "NaN === NaN");

    // Abstract equality coercions (ES5 11.9.3)
    assert_js(Value::Boolean(true), "null == undefined");
    assert_js(Value::Boolean(true), "undefined == null");
    assert_js(Value::Boolean(false), "null == 0");
    assert_js(Value::Boolean(false), "null == ''");
    assert_js(Value::Boolean(true), "0 == false");
    assert_js(Value::Boolean(true), "'' == false");
    assert_js(Value::Boolean(true), "'1' == 1");
    assert_js(Value::Boolean(false), "NaN == NaN");

    // NaN is not equal to itself
    assert_js(Value::Boolean(true), "NaN !== NaN");
}

#[test]
fn test_object_has_own_property() {
    assert_js(
        Value::Boolean(true),
        "var obj = { a: 1 }; obj.hasOwnProperty('a')",
    );
    assert_js(
        Value::Boolean(false),
        "var obj = { a: 1 }; obj.hasOwnProperty('b')",
    );
    assert_js(
        Value::Boolean(false),
        "var obj = {}; obj.hasOwnProperty('toString')",
    );
}

#[test]
fn test_object_to_string() {
    assert_js(
        Value::String(String::from("[object Object]")),
        "var obj = {}; obj.toString()",
    );
    assert_js(
        Value::String(String::from("[object Object]")),
        "({}).toString()",
    );
}

#[test]
fn test_function_expressions() {
    // Named function expressions are callable
    assert_js(
        Value::Number(5.0),
        "var result = (function() { return 5; })(); result",
    );
}

#[test]
fn test_arguments_object() {
    assert_js(
        Value::Number(6.0),
        "function sum() { var s = 0; for (var i = 0; i < arguments.length; i++) { s += arguments[i]; } return s; } sum(1, 2, 3)",
    );

    // arguments.length
    assert_js(
        Value::Number(3.0),
        "function f() { return arguments.length; } f(1, 2, 3)",
    );
}

#[test]
fn test_this_binding() {
    // Method call: this is the receiver
    assert_js(
        Value::Number(42.0),
        "var obj = { x: 42, getX: function() { return this.x; } }; obj.getX()",
    );
}

#[test]
fn test_prototype_chain_missing() {
    // Object instances don't have built-in prototype methods beyond what is implemented
    assert_js(
        Value::String(String::from("[object Object]")),
        "var o = {}; String(o)",
    );
}

#[test]
fn test_number_special_values() {
    assert_js(Value::Boolean(true), "isNaN(NaN)");
    assert_js(Value::Boolean(false), "isNaN(1)");
    assert_js(Value::Boolean(false), "isFinite(Infinity)");
    assert_js(Value::Boolean(true), "isFinite(1)");
    assert_js(Value::Boolean(false), "isFinite(NaN)");
}

#[test]
fn test_string_conversion() {
    assert_js(Value::String(String::from("42")), "String(42)");
    assert_js(Value::String(String::from("true")), "String(true)");
    assert_js(Value::String(String::from("null")), "String(null)");
    assert_js(
        Value::String(String::from("undefined")),
        "String(undefined)",
    );
}

#[test]
fn test_number_conversion() {
    assert_js(Value::Number(42.0), "Number('42')");
    assert_js(Value::Number(0.0), "Number(false)");
    assert_js(Value::Number(1.0), "Number(true)");
    assert_js(Value::Number(0.0), "Number(null)");
    assert_js(Value::Boolean(true), "isNaN(Number(undefined))");
    assert_js(Value::Boolean(true), "isNaN(Number('abc'))");
}

#[test]
fn test_boolean_conversion() {
    assert_js(Value::Boolean(false), "Boolean(0)");
    assert_js(Value::Boolean(false), "Boolean('')");
    assert_js(Value::Boolean(false), "Boolean(null)");
    assert_js(Value::Boolean(false), "Boolean(undefined)");
    assert_js(Value::Boolean(false), "Boolean(NaN)");
    assert_js(Value::Boolean(true), "Boolean(1)");
    assert_js(Value::Boolean(true), "Boolean('hello')");
    assert_js(Value::Boolean(true), "Boolean({})");
}

#[test]
fn test_array_literal() {
    assert_js(Value::Number(3.0), "[1, 2, 3].length");
    assert_js(Value::Number(2.0), "[1, 2, 3][1]");
    assert_js(Value::Undefined, "[1, 2, 3][5]");
}

#[test]
fn test_comma_operator() {
    // Comma operator evaluates all expressions, returns last
    assert_js(Value::Number(3.0), "(1, 2, 3)");
    assert_js(Value::Number(5.0), "var x = (1, 2, 5); x");
}

#[test]
fn test_delete_operator() {
    assert_js(
        Value::Boolean(false),
        "var obj = { a: 1 }; delete obj.a; obj.hasOwnProperty('a')",
    );
}

#[test]
fn test_in_operator() {
    assert_js(Value::Boolean(true), "'a' in { a: 1 }");
    assert_js(Value::Boolean(false), "'b' in { a: 1 }");
    assert_js(Value::Boolean(true), "0 in [1, 2, 3]");
}

#[test]
fn test_instanceof_operator() {
    // Basic instanceof with constructor functions
    assert_js(
        Value::Boolean(false),
        "function Foo() {} var f = {}; f instanceof Foo",
    );
}

#[test]
fn test_strict_equality_types() {
    // ES5 11.9.6 - test262 typeof/*.js series
    // Different types are never strictly equal
    assert_js(Value::Boolean(false), "1 === '1'");
    assert_js(Value::Boolean(false), "0 === false");
    assert_js(Value::Boolean(false), "0 === null");
    assert_js(Value::Boolean(false), "'' === false");
    assert_js(Value::Boolean(false), "'' === 0");
    assert_js(Value::Boolean(false), "null === undefined");
    // Same types and values are strictly equal
    assert_js(Value::Boolean(true), "1 === 1");
    assert_js(Value::Boolean(true), "'hello' === 'hello'");
    assert_js(Value::Boolean(true), "true === true");
    assert_js(Value::Boolean(true), "null === null");
    assert_js(Value::Boolean(true), "undefined === undefined");
    // NaN is not equal to itself
    assert_js(Value::Boolean(false), "NaN === NaN");
    assert_js(Value::Boolean(true), "NaN !== NaN");
    // Object identity - different objects with same content are not ===
    assert_js(Value::Boolean(false), "({}) === ({})");
    assert_js(Value::Boolean(false), "[] === []");
}

#[test]
fn test_addition_coercion_es5() {
    // ES5 11.6.1 - test262 S11.6.1_A3 series
    assert_js(Value::Number(2.0), "true + 1");
    assert_js(Value::Number(2.0), "1 + true");
    assert_js(Value::Number(1.0), "true + null");
    assert_js(Value::Number(1.0), "null + true");
    assert_js(Value::Number(0.0), "null + null");
    assert_js(Value::Boolean(true), "isNaN(undefined + 1)");
    assert_js(Value::Boolean(true), "isNaN(undefined + undefined)");
    // String coercion takes priority
    assert_js(Value::String("1true".into()), r#"'1' + true"#);
    assert_js(Value::String("1null".into()), r#"'1' + null"#);
    assert_js(Value::String("1undefined".into()), r#"'1' + undefined"#);
}

#[test]
fn test_division_ieee754() {
    // ES5 11.5.2 - test262 S11.5.2_A4 series
    assert_js(Value::Number(f64::INFINITY), "5 / 0");
    assert_js(Value::Number(f64::NEG_INFINITY), "-5 / 0");
    assert_js(Value::Boolean(true), "isNaN(0 / 0)");
    assert_js(Value::Number(0.0), "1 / Infinity");
    assert_js(Value::Number(0.0), "1 / -Infinity");
    assert_js(Value::Number(f64::INFINITY), "Infinity / 0");
    assert_js(Value::Boolean(true), "isNaN(Infinity / Infinity)");
    assert_js(Value::Boolean(false), "isFinite(1 / 0)");
    assert_js(Value::Boolean(true), "isFinite(1 / 2)");
}

#[test]
fn test_remainder_ieee754() {
    // ES5 11.5.3 - modulo edge cases
    assert_js(Value::Boolean(true), "isNaN(1 % 0)");
    assert_js(Value::Boolean(true), "isNaN(Infinity % 1)");
    assert_js(Value::Number(1.0), "1 % Infinity");
    assert_js(Value::Number(0.0), "0 % 1");
}

#[test]
fn test_for_in_enumeration() {
    // ES5 12.6.4 - test262 S12.6.4_A* series
    assert_js(
        Value::Number(3.0),
        r#"
        var obj = { a: 1, b: 2, c: 3 };
        var count = 0;
        for (var key in obj) { count++; }
        count
        "#,
    );
    assert_js(
        Value::Number(6.0),
        r#"
        var obj = { a: 1, b: 2, c: 3 };
        var sum = 0;
        for (var key in obj) { sum += obj[key]; }
        sum
        "#,
    );
    // for-in on non-object returns without iterating
    assert_js(
        Value::Number(0.0),
        r#"var count = 0; for (var k in null) { count++; } count"#,
    );
}

#[test]
fn test_try_catch_finally_es5() {
    // ES5 12.14 - test262 S12.14 series
    // catch receives thrown value
    assert_js(
        Value::Number(42.0),
        "try { throw 42; } catch (e) { e }",
    );
    // finally always runs
    assert_js(
        Value::Number(2.0),
        r#"
        var x = 0;
        try { x = 1; } finally { x = 2; }
        x
        "#,
    );
    // finally overrides return
    assert_js(
        Value::Number(2.0),
        "function f() { try { return 1; } finally { return 2; } } f()",
    );
    // catch then finally
    assert_js(
        Value::Number(3.0),
        r#"
        var x = 0;
        try { throw 1; } catch (e) { x = e + 1; } finally { x += 1; }
        x
        "#,
    );
    // re-throw from catch
    assert_js(
        Value::String("caught".into()),
        r#"
        var msg = '';
        try {
            try { throw 'inner'; } catch (e) { throw 'rethrown'; }
        } catch (e2) {
            msg = 'caught';
        }
        msg
        "#,
    );
}

#[test]
fn test_function_hoisting() {
    // ES5 10.5: function declarations are hoisted to the top of their scope
    assert_js(
        Value::Number(5.0),
        "var r = f(); function f() { return 5; } r",
    );
    assert_js(
        Value::Number(10.0),
        r#"
        function outer() {
            var r = inner();
            function inner() { return 10; }
            return r;
        }
        outer()
        "#,
    );
}

#[test]
fn test_var_hoisting() {
    // ES5 10.5: var declarations are hoisted (but not initialized)
    assert_js(Value::Undefined, "x; var x = 5");
    assert_js(
        Value::Number(5.0),
        "var x; x = 5; x",
    );
}

#[test]
fn test_arguments_callee_length() {
    // ES5 10.6: arguments object properties
    assert_js(
        Value::Number(3.0),
        "function f() { return arguments.length; } f(1, 2, 3)",
    );
    assert_js(
        Value::Number(0.0),
        "function f() { return arguments.length; } f()",
    );
    assert_js(
        Value::Number(99.0),
        "function f() { return arguments[0]; } f(99)",
    );
}

#[test]
fn test_string_number_comparison() {
    // ES5 11.8: comparison of string and number coerces string to number
    assert_js(Value::Boolean(true), r#""10" > 9"#);
    assert_js(Value::Boolean(false), r#""10" > 10"#);
    assert_js(Value::Boolean(true), r#""10" >= 10"#);
    assert_js(Value::Boolean(true), r#""5" < 10"#);
    // Pure string comparison is lexicographic
    assert_js(Value::Boolean(true), r#""10" < "9""#);
    assert_js(Value::Boolean(true), r#""b" > "a""#);
    assert_js(Value::Boolean(false), r#""z" < "a""#);
}
