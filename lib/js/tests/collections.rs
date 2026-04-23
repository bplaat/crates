/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Array and object data-structure tests.

mod common;

use common::assert_js;
use js::Value;

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
    assert_js(Value::Number(3.0), "[1, 2, 3].length");
    assert_js(Value::Number(0.0), "[].length");
    assert_js(Value::Number(5.0), "[10, 20, 30, 40, 50].length");
    assert_js(Value::Number(1.0), "['hello'].length");
    assert_js(Value::Number(4.0), "let arr = [1, 2, 3, 4]; arr.length");
    assert_js(Value::Number(2.0), "let arr = ['a', 'b']; arr.length");
    assert_js(
        Value::Number(10.0),
        "let arr = []; for (let i = 0; i < 10; i++) { arr[i] = i; } arr.length",
    );
    assert_js(
        Value::Number(6.0),
        "let arr = [1, 2, 3]; arr.push(4); arr.push(5); arr.push(6); arr.length",
    );
    assert_js(
        Value::Number(2.0),
        "let arr = [1, 2, 3, 4]; arr.length = 2; arr.length",
    );
    assert_js(
        Value::Number(15.0),
        "let arr = [1, 2, 3, 4, 5]; let sum = 0; for (let i = 0; i < arr.length; i++) { sum += arr[i]; } sum",
    );
    assert_js(
        Value::Number(120.0),
        "let arr = [1, 2, 3, 4, 5]; let product = 1; let i = 0; while (i < arr.length) { product *= arr[i]; i++; } product",
    );
}

#[test]
fn test_objects() {
    assert_js(
        Value::Number(25.0),
        "let obj = { a: 10, b: 15 }; obj.a + obj.b",
    );
    assert_js(
        Value::String(String::from("hello")),
        r#"let obj = { greeting: "hello" }; obj.greeting"#,
    );
    assert_js(
        Value::Number(50.0),
        "let obj = { x: 20, y: 30 }; obj.x + obj.y",
    );
    assert_js(Value::Undefined, "let obj = { a: 10 }; obj.b");
    assert_js(
        Value::Number(100.0),
        "let obj = { val: 100 }; let key = 'val'; obj[key]",
    );
    assert_js(
        Value::Number(200.0),
        "let obj = { val1: 100, val2: 200 }; let key = 'val2'; obj[key]",
    );
    assert_js(
        Value::Number(300.0),
        "let obj = { a: 100, b: 200 }; obj['a'] + obj['b']",
    );
}

#[test]
fn test_nested_arrays() {
    assert_js(
        Value::Number(5.0),
        "let arr = [[1, 2], [3, 4, 5]]; arr[1][2]",
    );
    assert_js(Value::Number(3.0), "let arr = [[1, 2, 3]]; arr[0].length");
    assert_js(
        Value::Number(8.0),
        "let arr = [1, [2, 3], 4, [5, 6]]; arr[1][0] + arr[3][1]",
    );
    assert_js(Value::Number(1.0), "let arr = [[[1]]]; arr[0][0][0]");
    assert_js(
        Value::Number(8.0),
        "let arr = []; arr[0] = [1, 2, 3]; arr[1] = [4, 5]; arr[0][2] + arr[1][1]",
    );
}

#[test]
fn test_nested_objects() {
    assert_js(
        Value::Number(15.0),
        "let obj = { a: { x: 10, y: 5 } }; obj.a.x + obj.a.y",
    );
    assert_js(
        Value::String(String::from("hello")),
        "let obj = { nested: { msg: 'hello' } }; obj.nested.msg",
    );
    assert_js(
        Value::Number(42.0),
        "let obj = { a: { b: { c: 42 } } }; obj.a.b.c",
    );
    assert_js(
        Value::Number(100.0),
        "let obj = { x: { y: { z: 100 } } }; obj['x']['y']['z']",
    );
    assert_js(
        Value::Number(25.0),
        "let obj = { p: { q: 10, r: 15 } }; obj['p'].q + obj.p['r']",
    );
}

#[test]
fn test_mixed_arrays_objects() {
    assert_js(
        Value::Number(30.0),
        "let data = [{x: 10}, {x: 20}]; data[0].x + data[1].x",
    );
    assert_js(
        Value::Number(15.0),
        "let data = { arr: [5, 10, 15] }; data.arr[2]",
    );
    assert_js(
        Value::Number(50.0),
        "let data = { items: [{val: 20}, {val: 30}] }; data.items[0].val + data.items[1].val",
    );
    assert_js(
        Value::Number(100.0),
        "let data = [{a: {b: 40}}, {a: {b: 60}}]; data[0].a.b + data[1].a.b",
    );
    assert_js(
        Value::Number(8.0),
        "let matrix = [{row: [1, 2, 3]}, {row: [4, 5, 6]}]; matrix[0].row[1] + matrix[1].row[2]",
    );
}

#[test]
fn test_array_mutation_complex() {
    assert_js(
        Value::Number(6.0),
        "let arr = [1, 2, 3]; arr[3] = 4; arr[4] = 5; arr[5] = 6; arr.length",
    );
    assert_js(
        Value::Number(100.0),
        "let arr = [1, 2, 3, 4, 5]; arr[2] = 100; arr[2]",
    );
    assert_js(
        Value::Number(3.0),
        "let arr = [1, 2, 3, 4, 5]; arr.length = 3; arr.length",
    );
    assert_js(
        Value::Undefined,
        "let arr = [1, 2, 3, 4, 5]; arr.length = 3; arr[4]",
    );
    assert_js(
        Value::Number(13.0),
        "let arr = [1, 2, 3]; for (let i = 0; i < 10; i++) { arr.push(i); } arr.length",
    );
}

#[test]
fn test_object_property_types() {
    assert_js(
        Value::Number(42.0),
        "let obj = { a: 42, b: 'text', c: true }; obj.a",
    );
    assert_js(
        Value::String(String::from("text")),
        "let obj = { a: 42, b: 'text', c: true }; obj.b",
    );
    assert_js(
        Value::Boolean(true),
        "let obj = { a: 42, b: 'text', c: true }; obj.c",
    );
    assert_js(
        Value::Number(15.0),
        "let obj = { x: 5, y: 10, getSum: 'x+y' }; obj.x + obj.y",
    );
    assert_js(Value::Undefined, "let obj = { a: 1 }; obj.nonexistent");
}

#[test]
fn test_array_object_iteration() {
    assert_js(
        Value::Number(15.0),
        "let arr = [1, 2, 3, 4, 5]; let sum = 0; for (let i = 0; i < arr.length; i++) { sum += arr[i]; } sum",
    );
    assert_js(
        Value::Number(21.0),
        "let obj = { a: 5, b: 10, c: 6 }; obj.a + obj.b + obj.c",
    );
    assert_js(
        Value::Number(15.0),
        "let arr = [{val: 1}, {val: 2}, {val: 3}, {val: 4}, {val: 5}]; let sum = 0; for (let i = 0; i < arr.length; i++) { sum += arr[i].val; } sum",
    );
    assert_js(
        Value::Number(150.0),
        "let obj = { nums: [10, 20, 30, 40, 50] }; let sum = 0; for (let i = 0; i < obj.nums.length; i++) { sum += obj.nums[i]; } sum",
    );
}

#[test]
fn test_array_methods_chain() {
    assert_js(
        Value::Number(6.0),
        "let arr = [1, 2]; arr.push(3); arr.push(4); arr.push(5); arr.push(6); arr.length",
    );
    assert_js(
        Value::Number(10.0),
        "let arr = []; for (let i = 1; i <= 10; i++) { arr.push(i); } arr.length",
    );
    assert_js(
        Value::Number(55.0),
        "let arr = []; for (let i = 1; i <= 10; i++) { arr.push(i); } let sum = 0; for (let j = 0; j < arr.length; j++) { sum += arr[j]; } sum",
    );
}

#[test]
fn test_object_modification() {
    assert_js(
        Value::Number(100.0),
        "let obj = { a: 10, b: 20 }; obj.a = 100; obj.a",
    );
    assert_js(
        Value::Number(130.0),
        "let obj = { x: 10, y: 20 }; obj.x = 100; obj.y = 30; obj.x + obj.y",
    );
    assert_js(
        Value::Number(999.0),
        "let obj = { val: 1 }; obj['val'] = 999; obj.val",
    );
}

#[test]
fn test_complex_data_structures() {
    assert_js(
        Value::Number(55.0),
        "let db = { users: [{id: 1, age: 25}, {id: 2, age: 30}, {id: 3, age: 13}] }; db.users[0].age + db.users[1].age",
    );
    assert_js(
        Value::Number(180.0),
        "let data = { matrix: [[10, 20], [30, 40], [50, 60], [70, 80]] }; data.matrix[0][0] + data.matrix[1][1] + data.matrix[2][0] + data.matrix[3][1]",
    );
    assert_js(
        Value::Number(24.0),
        "let config = { levels: [1, 2, 3, 4], multiplier: 1 }; config.levels[0] * config.levels[1] * config.levels[2] * config.levels[3]",
    );
    assert_js(
        Value::Number(21.0),
        "let obj = { a: { arr: [1, 2, 3] }, b: { arr: [4, 5, 6] } }; let s = 0; for (let i = 0; i < obj.a.arr.length; i++) { s += obj.a.arr[i]; } for (let i = 0; i < obj.b.arr.length; i++) { s += obj.b.arr[i]; } s",
    );
}

#[test]
fn test_empty_structures() {
    assert_js(Value::Number(0.0), "[].length");
    assert_js(Value::Number(0.0), "let arr = []; arr.length");
    assert_js(Value::Undefined, "let obj = {}; obj.a");
    assert_js(
        Value::Number(0.0),
        "let arr = [1, 2, 3]; arr.length = 0; arr.length",
    );
}

#[test]
fn test_array_sparse() {
    assert_js(Value::Undefined, "let arr = []; arr[10] = 42; arr[0]");
    assert_js(Value::Number(42.0), "let arr = []; arr[10] = 42; arr[10]");
    assert_js(
        Value::Number(11.0),
        "let arr = []; arr[10] = 42; arr.length",
    );
    assert_js(
        Value::Number(100.0),
        "let arr = []; arr[0] = 50; arr[5] = 50; arr[0] + arr[5]",
    );
}
