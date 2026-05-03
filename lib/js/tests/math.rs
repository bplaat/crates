/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Math object tests (Phase 5).
//! Test cases modeled after tc39/test262 test/built-ins/Math/.

mod common;

use common::assert_js;
use js::Value;

fn approx(v: f64) -> Value {
    Value::Number(v)
}

fn bool_v(v: bool) -> Value {
    Value::Boolean(v)
}

#[test]
fn test_math_constants() {
    assert_js(approx(std::f64::consts::PI), "Math.PI");
    assert_js(approx(std::f64::consts::E), "Math.E");
    assert_js(approx(std::f64::consts::LN_2), "Math.LN2");
    assert_js(approx(std::f64::consts::LN_10), "Math.LN10");
    assert_js(approx(std::f64::consts::LOG2_E), "Math.LOG2E");
    assert_js(approx(std::f64::consts::LOG10_E), "Math.LOG10E");
    assert_js(approx(std::f64::consts::SQRT_2), "Math.SQRT2");
    assert_js(approx(1.0 / std::f64::consts::SQRT_2), "Math.SQRT1_2");
}

#[test]
fn test_math_rounding() {
    assert_js(approx(4.0), "Math.floor(4.7)");
    assert_js(approx(-5.0), "Math.floor(-4.1)");
    assert_js(approx(5.0), "Math.ceil(4.1)");
    assert_js(approx(-4.0), "Math.ceil(-4.7)");
    assert_js(approx(5.0), "Math.round(4.5)");
    assert_js(approx(4.0), "Math.round(4.4)");
    assert_js(approx(-4.0), "Math.round(-4.5)");
    assert_js(approx(4.0), "Math.trunc(4.9)");
    assert_js(approx(-4.0), "Math.trunc(-4.9)");
}

#[test]
fn test_math_abs() {
    assert_js(approx(5.0), "Math.abs(-5)");
    assert_js(approx(5.0), "Math.abs(5)");
    assert_js(approx(0.0), "Math.abs(0)");
    assert_js(Value::Boolean(true), "isNaN(Math.abs(NaN))");
}

#[test]
fn test_math_min_max() {
    assert_js(approx(1.0), "Math.min(1, 2, 3)");
    assert_js(approx(3.0), "Math.max(1, 2, 3)");
    assert_js(approx(f64::INFINITY), "Math.min()");
    assert_js(approx(f64::NEG_INFINITY), "Math.max()");
    assert_js(Value::Boolean(true), "isNaN(Math.min(1, NaN, 3))");
}

#[test]
fn test_math_pow_sqrt() {
    assert_js(approx(8.0), "Math.pow(2, 3)");
    assert_js(approx(1.0), "Math.pow(5, 0)");
    assert_js(approx(2.0), "Math.sqrt(4)");
    assert_js(Value::Boolean(true), "isNaN(Math.sqrt(-1))");
}

#[test]
fn test_math_log() {
    assert_js(approx(0.0), "Math.log(1)");
    assert_js(approx(1.0), "Math.log(Math.E)");
    assert_js(approx(0.0), "Math.log2(1)");
    assert_js(approx(1.0), "Math.log2(2)");
    assert_js(approx(0.0), "Math.log10(1)");
    assert_js(approx(1.0), "Math.log10(10)");
}

#[test]
fn test_math_exp() {
    assert_js(approx(1.0), "Math.exp(0)");
    assert_js(approx(std::f64::consts::E), "Math.exp(1)");
}

#[test]
fn test_math_sign() {
    assert_js(approx(1.0), "Math.sign(42)");
    assert_js(approx(-1.0), "Math.sign(-42)");
    assert_js(approx(0.0), "Math.sign(0)");
    assert_js(Value::Boolean(true), "isNaN(Math.sign(NaN))");
}

#[test]
fn test_math_trig() {
    assert_js(approx(0.0), "Math.sin(0)");
    assert_js(approx(1.0), "Math.cos(0)");
    assert_js(approx(0.0), "Math.tan(0)");
    assert_js(approx(0.0), "Math.asin(0)");
    assert_js(approx(0.0), "Math.acos(1)");
    assert_js(approx(0.0), "Math.atan(0)");
    assert_js(approx(std::f64::consts::FRAC_PI_4), "Math.atan2(1, 1)");
}

#[test]
fn test_math_hypot() {
    assert_js(approx(5.0), "Math.hypot(3, 4)");
    assert_js(approx(0.0), "Math.hypot()");
}

#[test]
fn test_math_cbrt() {
    assert_js(approx(2.0), "Math.cbrt(8)");
    assert_js(approx(-2.0), "Math.cbrt(-8)");
}

#[test]
fn test_math_random() {
    // Just check that it returns a number between 0 and 1
    assert_js(bool_v(true), "Math.random() >= 0 && Math.random() < 1");
}

#[test]
fn test_math_clz32() {
    assert_js(approx(24.0), "Math.clz32(255)");
    assert_js(approx(32.0), "Math.clz32(0)");
}

#[test]
fn test_math_imul() {
    assert_js(approx(6.0), "Math.imul(2, 3)");
    assert_js(approx(-2.0), "Math.imul(0xffffffff, 2)");
}
