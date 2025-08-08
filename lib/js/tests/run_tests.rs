/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Run JS test assertions.

use js::Context;

#[test]
fn test_run_tests() {
    fn assert_js(expected: i64, script: &str) {
        let mut context = Context::new();
        let result = context.eval(script).unwrap();
        assert_eq!(expected, result);
    }

    assert_js(0, "0");
    assert_js(42, "42");
    assert_js(21, "5+20-4");
    assert_js(41, " 12 + 34 - 5 ");
    assert_js(47, "5+6*7");
    assert_js(15, "5*(9-6)");
    assert_js(4, "(3 + 5) / 2");
    assert_js(16, "2 ** 4");
    assert_js(0, "36 / 0");
    assert_js(2, "20 % 3");
    assert_js(34, "--34");
    assert_js(-40, "---40");
    assert_js(1, "5 & 3");
    assert_js(7, "5 | 3");
    assert_js(6, "5 ^ 3");
    assert_js(16, "8 << 1");
    assert_js(4, "8 >> 1");
    assert_js(0, "8 >> 4");
    assert_js(4, "8 >>> 1");
    assert_js(0, "8 >>> 4");

    assert_js(40, "20;30;40");
    assert_js(91, "34,  48,91");
    assert_js(10, "a = 10");
    assert_js(40, "a = 5,a * 8");
    assert_js(100, "a=10;b = 90;a + b");
    assert_js(40, "a=  b= 20,  a+b");
}
