/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! for-in and for-of iteration tests.

mod common;

use common::assert_js;
use js::Value;

#[test]
fn test_for_in_objects() {
    assert_js(
        Value::Number(3.0),
        "let obj = {a: 1, b: 2, c: 3}; let count = 0; for (let key in obj) { count++; } count",
    );
    assert_js(
        Value::String(String::from("abc")),
        "let obj = {a: 1, b: 2, c: 3}; let keys = ''; for (let key in obj) { keys += key; } keys",
    );
    assert_js(
        Value::Number(6.0),
        "let obj = {a: 1, b: 2, c: 3}; let sum = 0; for (let key in obj) { sum += obj[key]; } sum",
    );
    assert_js(
        Value::Number(0.0),
        "let obj = {}; let count = 0; for (let key in obj) { count++; } count",
    );
    assert_js(
        Value::Number(2.0),
        "let nested = {x: {a: 1}, y: {b: 2}}; let count = 0; for (let key in nested) { count++; } count",
    );
}

#[test]
fn test_for_in_arrays() {
    assert_js(
        Value::Number(3.0),
        "let arr = [10, 20, 30]; let count = 0; for (let i in arr) { count++; } count",
    );
    assert_js(
        Value::String(String::from("012")),
        "let arr = [10, 20, 30]; let indices = ''; for (let i in arr) { indices += i; } indices",
    );
    assert_js(
        Value::Number(60.0),
        "let arr = [10, 20, 30]; let sum = 0; for (let i in arr) { sum += arr[i]; } sum",
    );
    assert_js(
        Value::Number(2.0),
        "let arr = []; arr[0] = 'a'; arr[2] = 'c'; let count = 0; for (let i in arr) { count++; } count",
    );
}

#[test]
fn test_for_in_with_break() {
    assert_js(
        Value::Number(1.0),
        "let obj = {a: 1, b: 2, c: 3}; let count = 0; for (let key in obj) { count++; break; } count",
    );
    assert_js(
        Value::Number(2.0),
        "let arr = [10, 20, 30]; let count = 0; for (let i in arr) { count++; if (count == 2) break; } count",
    );
}

#[test]
fn test_for_in_with_continue() {
    assert_js(
        Value::Number(3.0),
        "let obj = {a: 1, b: 2, c: 3}; let count = 0; for (let key in obj) { count++; continue; } count",
    );
    assert_js(
        Value::Number(40.0),
        "let arr = [10, 20, 30]; let sum = 0; for (let i in arr) { if (i == 1) continue; sum += arr[i]; } sum",
    );
}

#[test]
fn test_for_of_arrays() {
    assert_js(
        Value::Number(3.0),
        "let arr = [10, 20, 30]; let count = 0; for (let val of arr) { count++; } count",
    );
    assert_js(
        Value::Number(60.0),
        "let arr = [10, 20, 30]; let sum = 0; for (let val of arr) { sum += val; } sum",
    );
    assert_js(
        Value::String(String::from("abc")),
        "let arr = ['a', 'b', 'c']; let str = ''; for (let char of arr) { str += char; } str",
    );
    assert_js(
        Value::Number(0.0),
        "let arr = []; let count = 0; for (let val of arr) { count++; } count",
    );
    assert_js(
        Value::Number(3.0),
        "let arr = [1, 2, 3]; let count = 0; for (const val of arr) { count++; } count",
    );
}

#[test]
fn test_for_of_nested_arrays() {
    assert_js(
        Value::Number(10.0),
        "let arr = [[1, 2], [3, 4]]; let sum = 0; for (let inner of arr) { for (let val of inner) { sum += val; } } sum",
    );
    assert_js(
        Value::Number(3.0),
        "let arr = [[10], [20, 30]]; let count = 0; for (let inner of arr) { count += inner.length; } count",
    );
}

#[test]
fn test_for_of_with_break() {
    assert_js(
        Value::Number(1.0),
        "let arr = [10, 20, 30]; let count = 0; for (let val of arr) { count++; break; } count",
    );
    assert_js(
        Value::Number(30.0),
        "let arr = [10, 20, 30]; let sum = 0; for (let val of arr) { sum += val; if (sum > 25) break; } sum",
    );
}

#[test]
fn test_for_of_with_continue() {
    assert_js(
        Value::Number(3.0),
        "let arr = [10, 20, 30]; let count = 0; for (let val of arr) { count++; continue; } count",
    );
    assert_js(
        Value::Number(40.0),
        "let arr = [10, 20, 30]; let sum = 0; for (let val of arr) { if (val == 20) continue; sum += val; } sum",
    );
}

#[test]
fn test_for_in_of_var_declaration() {
    assert_js(
        Value::Number(3.0),
        "let obj = {a: 1, b: 2, c: 3}; let count = 0; for (var key in obj) { count++; } count",
    );
    assert_js(
        Value::Number(3.0),
        "let arr = [10, 20, 30]; let count = 0; for (var i in arr) { count++; } count",
    );
    assert_js(
        Value::Number(3.0),
        "let arr = [10, 20, 30]; let count = 0; for (var val of arr) { count++; } count",
    );
}

#[test]
fn test_labeled_for_in_of() {
    assert_js(
        Value::Number(1.0),
        "let obj = {a: 1, b: 2}; let count = 0; outer: for (let key in obj) { count++; break outer; } count",
    );
    assert_js(
        Value::Number(2.0),
        "let arr = [1, 2, 3]; let count = 0; loop: for (let val of arr) { count++; if (count == 2) break loop; } count",
    );
}
