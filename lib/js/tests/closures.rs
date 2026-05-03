/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Closure and lexical scope tests.

mod common;

use common::assert_js;
use js::Value;

#[test]
fn test_basic_closures() {
    // Function returned from outer captures outer's variable
    assert_js(
        Value::Number(5.0),
        "function outer() { let n = 5; function inner() { return n; } return inner; }
         var f = outer(); f()",
    );

    // Counter closure: closed-over var mutates across calls
    assert_js(
        Value::Number(3.0),
        "function counter() { var n = 0; return function() { return ++n; }; }
         var c = counter(); c(); c(); c()",
    );

    // IIFE captures enclosing scope
    assert_js(
        Value::Number(42.0),
        "var x = 42; var result = (function() { return x; })(); result",
    );

    // Nested closures three levels deep
    assert_js(
        Value::Number(30.0),
        "function outer() {
           let x = 10;
           function middle() {
             let y = 20;
             function inner() { return x + y; }
             return inner();
           }
           return middle();
         }
         outer()",
    );

    // Closure reads outer let after outer returns
    assert_js(
        Value::Number(10.0),
        "function make() { let v = 10; return function() { return v; }; }
         make()()",
    );
}

#[test]
fn test_closure_mutation() {
    // Closure mutates outer variable
    assert_js(
        Value::Number(99.0),
        "function make() {
           var v = 0;
           function set(x) { v = x; }
           function get() { return v; }
           return { set: set, get: get };
         }
         var obj = make();
         obj.set(99);
         obj.get()",
    );

    // Multiple closures share same captured variable
    assert_js(
        Value::Number(2.0),
        "function make() {
           var count = 0;
           function inc() { count++; }
           function get() { return count; }
           return { inc: inc, get: get };
         }
         var m = make();
         m.inc();
         m.inc();
         m.get()",
    );
}

#[test]
fn test_closure_shadowing() {
    // Inner let shadows outer let; closure returns inner's copy
    assert_js(
        Value::Number(20.0),
        "function outer() {
           let x = 5;
           function inner() { let x = 20; return x; }
           return inner();
         }
         outer()",
    );

    // After inner returns, outer's x is unchanged
    assert_js(
        Value::Number(5.0),
        "function outer() {
           let x = 5;
           function inner() { let x = 20; return x; }
           inner();
           return x;
         }
         outer()",
    );
}
