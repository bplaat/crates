/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::{make_obj, native};
use crate::value::Value;

pub(super) fn register(env: &Rc<RefCell<IndexMap<String, Value>>>) {
    env.borrow_mut().insert(
        "Math".to_string(),
        make_obj(&[
            ("PI", Value::Number(std::f64::consts::PI)),
            ("E", Value::Number(std::f64::consts::E)),
            ("LN2", Value::Number(std::f64::consts::LN_2)),
            ("LN10", Value::Number(std::f64::consts::LN_10)),
            ("LOG2E", Value::Number(std::f64::consts::LOG2_E)),
            ("LOG10E", Value::Number(std::f64::consts::LOG10_E)),
            ("SQRT2", Value::Number(std::f64::consts::SQRT_2)),
            ("SQRT1_2", Value::Number(1.0 / std::f64::consts::SQRT_2)),
            (
                "abs",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().abs()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "ceil",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().ceil()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "floor",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().floor()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "round",
                native(|a| {
                    // JS Math.round: round half toward +infinity (not away from zero)
                    Value::Number(
                        a.first()
                            .map(|v| (v.to_number() + 0.5).floor())
                            .unwrap_or(f64::NAN),
                    )
                }),
            ),
            (
                "trunc",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().trunc()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "sign",
                native(|a| {
                    let n = a.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
                    if n.is_nan() {
                        Value::Number(f64::NAN)
                    } else if n > 0.0 {
                        Value::Number(1.0)
                    } else if n < 0.0 {
                        Value::Number(-1.0)
                    } else {
                        Value::Number(0.0)
                    }
                }),
            ),
            (
                "sqrt",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().sqrt()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "cbrt",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().cbrt()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "exp",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().exp()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "log",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().ln()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "log2",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().log2()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "log10",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().log10()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "pow",
                native(|a| {
                    let base = a.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
                    let exp = a.get(1).map(|v| v.to_number()).unwrap_or(f64::NAN);
                    Value::Number(base.powf(exp))
                }),
            ),
            (
                "max",
                native(|a| {
                    if a.is_empty() {
                        return Value::Number(f64::NEG_INFINITY);
                    }
                    let mut m = f64::NEG_INFINITY;
                    for v in a {
                        let n = v.to_number();
                        if n.is_nan() {
                            return Value::Number(f64::NAN);
                        }
                        if n > m {
                            m = n;
                        }
                    }
                    Value::Number(m)
                }),
            ),
            (
                "min",
                native(|a| {
                    if a.is_empty() {
                        return Value::Number(f64::INFINITY);
                    }
                    let mut m = f64::INFINITY;
                    for v in a {
                        let n = v.to_number();
                        if n.is_nan() {
                            return Value::Number(f64::NAN);
                        }
                        if n < m {
                            m = n;
                        }
                    }
                    Value::Number(m)
                }),
            ),
            (
                "random",
                native(|_| {
                    use std::cell::Cell;
                    use std::time::{SystemTime, UNIX_EPOCH};
                    thread_local! { static STATE: Cell<u64> = const { Cell::new(0) }; }
                    STATE.with(|s| {
                        let mut v = s.get();
                        if v == 0 {
                            let seed = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .map(|d| d.subsec_nanos())
                                .unwrap_or(12345);
                            v = seed as u64 ^ 0xdeadbeef;
                        }
                        v ^= v << 13;
                        v ^= v >> 7;
                        v ^= v << 17;
                        s.set(v);
                        Value::Number((v as f64) / (u64::MAX as f64))
                    })
                }),
            ),
            (
                "sin",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().sin()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "cos",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().cos()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "tan",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().tan()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "asin",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().asin()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "acos",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().acos()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "atan",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().atan()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "atan2",
                native(|a| {
                    let y = a.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
                    let x = a.get(1).map(|v| v.to_number()).unwrap_or(f64::NAN);
                    Value::Number(y.atan2(x))
                }),
            ),
            (
                "sinh",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().sinh()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "cosh",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().cosh()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "tanh",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().tanh()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "asinh",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().asinh()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "acosh",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().acosh()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "atanh",
                native(|a| {
                    Value::Number(a.first().map(|v| v.to_number().atanh()).unwrap_or(f64::NAN))
                }),
            ),
            (
                "hypot",
                native(|a| {
                    let sum: f64 = a
                        .iter()
                        .map(|v| {
                            let n = v.to_number();
                            n * n
                        })
                        .sum();
                    Value::Number(sum.sqrt())
                }),
            ),
            (
                "clz32",
                native(|a| {
                    let n = a.first().map(|v| v.to_number() as u32).unwrap_or(0);
                    Value::Number(n.leading_zeros() as f64)
                }),
            ),
            (
                "imul",
                native(|a| {
                    let x = a.first().map(|v| v.to_number() as i32).unwrap_or(0);
                    let y = a.get(1).map(|v| v.to_number() as i32).unwrap_or(0);
                    Value::Number((x.wrapping_mul(y)) as f64)
                }),
            ),
            (
                "fround",
                native(|a| {
                    let n = a.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
                    Value::Number((n as f32) as f64)
                }),
            ),
        ]),
    );
}
