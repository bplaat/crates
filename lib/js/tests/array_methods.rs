/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Array prototype method tests (Phase 4).
//! Test cases modeled after tc39/test262 test/built-ins/Array/prototype/.

mod common;

use common::assert_js;
use js::Value;

fn arr(v: Vec<Value>) -> Value {
    Value::Array(js::ArrayValue {
        elements: std::rc::Rc::new(std::cell::RefCell::new(v)),
    })
}

fn n(v: f64) -> Value {
    Value::Number(v)
}

fn s(v: &str) -> Value {
    Value::String(v.to_string())
}

// MARK: Mutating methods

#[test]
fn test_push_pop() {
    assert_js(Value::Number(3.0), r#"let a = [1, 2]; a.push(3); a.length"#);
    assert_js(n(3.0), r#"let a = [1, 2, 3]; a.pop()"#);
    assert_js(
        Value::Number(2.0),
        r#"let a = [1, 2, 3]; a.pop(); a.length"#,
    );
    assert_js(Value::Undefined, r#"let a = []; a.pop()"#);
}

#[test]
fn test_shift_unshift() {
    assert_js(n(1.0), r#"let a = [1, 2, 3]; a.shift()"#);
    assert_js(
        Value::Number(2.0),
        r#"let a = [1, 2, 3]; a.shift(); a.length"#,
    );
    assert_js(Value::Number(4.0), r#"let a = [3, 4]; a.unshift(1, 2)"#);
    assert_js(
        arr(vec![n(1.0), n(2.0), n(3.0), n(4.0)]),
        r#"let a = [3, 4]; a.unshift(1, 2); a"#,
    );
}

#[test]
fn test_splice() {
    assert_js(arr(vec![n(2.0)]), r#"let a = [1, 2, 3]; a.splice(1, 1)"#);
    assert_js(
        arr(vec![n(1.0), n(3.0)]),
        r#"let a = [1, 2, 3]; a.splice(1, 1); a"#,
    );
    assert_js(
        arr(vec![n(1.0), s("x"), s("y"), n(3.0)]),
        r#"let a = [1, 2, 3]; a.splice(1, 1, "x", "y"); a"#,
    );
}

#[test]
fn test_reverse() {
    assert_js(arr(vec![n(3.0), n(2.0), n(1.0)]), r#"[1, 2, 3].reverse()"#);
    assert_js(arr(vec![]), r#"[].reverse()"#);
}

#[test]
fn test_sort_default() {
    // Default sort is lexicographic
    assert_js(
        arr(vec![n(1.0), n(10.0), n(2.0), n(9.0)]),
        r#"[10, 2, 1, 9].sort()"#,
    );
}

#[test]
fn test_sort_comparator() {
    assert_js(
        arr(vec![n(1.0), n(2.0), n(9.0), n(10.0)]),
        r#"[10, 2, 1, 9].sort(function(a, b) { return a - b; })"#,
    );
}

#[test]
fn test_fill() {
    assert_js(arr(vec![n(0.0), n(0.0), n(0.0)]), r#"[1, 2, 3].fill(0)"#);
    assert_js(arr(vec![n(1.0), n(7.0), n(7.0)]), r#"[1, 2, 3].fill(7, 1)"#);
    assert_js(
        arr(vec![n(1.0), n(7.0), n(3.0)]),
        r#"[1, 2, 3].fill(7, 1, 2)"#,
    );
}

// MARK: Non-mutating methods

#[test]
fn test_slice() {
    assert_js(arr(vec![n(2.0), n(3.0)]), r#"[1, 2, 3].slice(1)"#);
    assert_js(arr(vec![n(2.0)]), r#"[1, 2, 3].slice(1, 2)"#);
    assert_js(arr(vec![n(2.0), n(3.0)]), r#"[1, 2, 3].slice(-2)"#);
    assert_js(arr(vec![n(1.0), n(2.0), n(3.0)]), r#"[1, 2, 3].slice()"#);
}

#[test]
fn test_concat() {
    assert_js(
        arr(vec![n(1.0), n(2.0), n(3.0), n(4.0)]),
        r#"[1, 2].concat([3, 4])"#,
    );
    assert_js(
        arr(vec![n(1.0), n(2.0), n(3.0), n(4.0), n(5.0)]),
        r#"[1, 2].concat([3], [4, 5])"#,
    );
    assert_js(arr(vec![n(1.0), n(2.0), n(3.0)]), r#"[1, 2].concat(3)"#);
}

#[test]
fn test_join() {
    assert_js(s("1,2,3"), r#"[1, 2, 3].join()"#);
    assert_js(s("1-2-3"), r#"[1, 2, 3].join("-")"#);
    assert_js(s("123"), r#"[1, 2, 3].join("")"#);
}

#[test]
fn test_index_of_last_index_of() {
    assert_js(Value::Number(1.0), r#"[1, 2, 3, 2].indexOf(2)"#);
    assert_js(Value::Number(-1.0), r#"[1, 2, 3].indexOf(99)"#);
    assert_js(Value::Number(3.0), r#"[1, 2, 3, 2].lastIndexOf(2)"#);
}

#[test]
fn test_includes() {
    assert_js(Value::Boolean(true), r#"[1, 2, 3].includes(2)"#);
    assert_js(Value::Boolean(false), r#"[1, 2, 3].includes(99)"#);
}

#[test]
fn test_flat() {
    assert_js(
        arr(vec![n(1.0), n(2.0), n(3.0), n(4.0)]),
        r#"[1, [2, 3], 4].flat()"#,
    );
    assert_js(
        arr(vec![n(1.0), n(2.0), n(3.0), n(4.0)]),
        r#"[1, [2, [3]], 4].flat(2)"#,
    );
}

#[test]
fn test_at() {
    assert_js(n(1.0), r#"[1, 2, 3].at(0)"#);
    assert_js(n(3.0), r#"[1, 2, 3].at(-1)"#);
    assert_js(Value::Undefined, r#"[1, 2, 3].at(99)"#);
}

// MARK: Iteration methods

#[test]
fn test_for_each() {
    assert_js(
        Value::Number(6.0),
        r#"let sum = 0; [1, 2, 3].forEach(function(x) { sum += x; }); sum"#,
    );
}

#[test]
fn test_map() {
    assert_js(
        arr(vec![n(2.0), n(4.0), n(6.0)]),
        r#"[1, 2, 3].map(function(x) { return x * 2; })"#,
    );
    assert_js(
        arr(vec![n(2.0), n(4.0), n(6.0)]),
        r#"[1, 2, 3].map(x => x * 2)"#,
    );
}

#[test]
fn test_filter() {
    assert_js(
        arr(vec![n(2.0), n(4.0)]),
        r#"[1, 2, 3, 4, 5].filter(x => x % 2 === 0)"#,
    );
    assert_js(arr(vec![]), r#"[1, 3, 5].filter(x => x % 2 === 0)"#);
}

#[test]
fn test_reduce() {
    assert_js(
        n(6.0),
        r#"[1, 2, 3].reduce(function(acc, x) { return acc + x; }, 0)"#,
    );
    assert_js(n(6.0), r#"[1, 2, 3].reduce((acc, x) => acc + x)"#);
    assert_js(s("abc"), r#"["a","b","c"].reduce((acc, x) => acc + x, "")"#);
}

#[test]
fn test_reduce_right() {
    assert_js(
        s("cba"),
        r#"["a","b","c"].reduceRight((acc, x) => acc + x)"#,
    );
}

#[test]
fn test_some_every() {
    assert_js(Value::Boolean(true), r#"[1, 2, 3].some(x => x > 2)"#);
    assert_js(Value::Boolean(false), r#"[1, 2, 3].some(x => x > 5)"#);
    assert_js(Value::Boolean(true), r#"[2, 4, 6].every(x => x % 2 === 0)"#);
    assert_js(
        Value::Boolean(false),
        r#"[2, 3, 6].every(x => x % 2 === 0)"#,
    );
}

#[test]
fn test_find_find_index() {
    assert_js(n(3.0), r#"[1, 2, 3, 4].find(x => x > 2)"#);
    assert_js(Value::Undefined, r#"[1, 2, 3].find(x => x > 9)"#);
    assert_js(n(2.0), r#"[1, 2, 3, 4].findIndex(x => x > 2)"#);
    assert_js(Value::Number(-1.0), r#"[1, 2, 3].findIndex(x => x > 9)"#);
}

#[test]
fn test_flat_map() {
    assert_js(
        arr(vec![n(1.0), n(1.0), n(2.0), n(2.0), n(3.0), n(3.0)]),
        r#"[1, 2, 3].flatMap(x => [x, x])"#,
    );
}

// MARK: Static methods

#[test]
fn test_array_is_array() {
    assert_js(Value::Boolean(true), r#"Array.isArray([1, 2, 3])"#);
    assert_js(Value::Boolean(false), r#"Array.isArray("hello")"#);
    assert_js(Value::Boolean(false), r#"Array.isArray(42)"#);
}

#[test]
fn test_array_from() {
    assert_js(
        arr(vec![n(1.0), n(2.0), n(3.0)]),
        r#"Array.from([1, 2, 3])"#,
    );
}

#[test]
fn test_array_of() {
    assert_js(arr(vec![n(1.0), n(2.0), n(3.0)]), r#"Array.of(1, 2, 3)"#);
}
