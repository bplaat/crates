/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::{make_obj, native};
use crate::interpreter::Control;
use crate::value::{ArrayValue, Value};

pub(crate) fn call_method(
    arr: ArrayValue,
    method: &str,
    args: Vec<Value>,
    call_fn: &mut impl FnMut(Value, Value, Vec<Value>) -> Result<Value, Control>,
) -> Option<Result<Value, Control>> {
    call_method_inner(arr, method, args, call_fn).transpose()
}

fn call_method_inner(
    arr: ArrayValue,
    method: &str,
    args: Vec<Value>,
    call_fn: &mut impl FnMut(Value, Value, Vec<Value>) -> Result<Value, Control>,
) -> Result<Option<Value>, Control> {
    match method {
        "push" => {
            let mut borrowed = arr.borrow_mut();
            for val in args {
                borrowed.push(val);
            }
            Ok(Some(Value::Number(borrowed.len() as f64)))
        }
        "pop" => {
            let val = arr.borrow_mut().pop().unwrap_or(Value::Undefined);
            Ok(Some(val))
        }
        "shift" => {
            let mut borrowed = arr.borrow_mut();
            if borrowed.is_empty() {
                Ok(Some(Value::Undefined))
            } else {
                Ok(Some(borrowed.remove(0)))
            }
        }
        "unshift" => {
            let mut borrowed = arr.borrow_mut();
            for (i, val) in args.into_iter().enumerate() {
                borrowed.insert(i, val);
            }
            Ok(Some(Value::Number(borrowed.len() as f64)))
        }
        "length" => Ok(Some(Value::Number(arr.borrow().len() as f64))),
        "slice" => {
            let borrowed = arr.borrow();
            let len = borrowed.len();
            let start = args
                .first()
                .map(|v| {
                    let n = v.to_number() as i64;
                    if n < 0 { (len as i64 + n).max(0) as usize } else { (n as usize).min(len) }
                })
                .unwrap_or(0);
            let end = args
                .get(1)
                .map(|v| {
                    let n = v.to_number() as i64;
                    if n < 0 { (len as i64 + n).max(0) as usize } else { (n as usize).min(len) }
                })
                .unwrap_or(len);
            let result: Vec<Value> =
                if start <= end { borrowed[start..end].to_vec() } else { vec![] };
            Ok(Some(Value::Array(ArrayValue {
                elements: Rc::new(RefCell::new(result)),
            })))
        }
        "splice" => {
            let mut borrowed = arr.borrow_mut();
            let len = borrowed.len();
            let start = args
                .first()
                .map(|v| {
                    let n = v.to_number() as i64;
                    if n < 0 { (len as i64 + n).max(0) as usize } else { (n as usize).min(len) }
                })
                .unwrap_or(0);
            let delete_count = args
                .get(1)
                .map(|v| (v.to_number() as usize).min(len - start))
                .unwrap_or(len - start);
            let removed: Vec<Value> = borrowed.drain(start..start + delete_count).collect();
            for (i, val) in args.into_iter().skip(2).enumerate() {
                borrowed.insert(start + i, val);
            }
            Ok(Some(Value::Array(ArrayValue {
                elements: Rc::new(RefCell::new(removed)),
            })))
        }
        "concat" => {
            let mut result = arr.borrow().clone();
            for arg in args {
                match arg {
                    Value::Array(other) => result.extend(other.borrow().clone()),
                    val => result.push(val),
                }
            }
            Ok(Some(Value::Array(ArrayValue {
                elements: Rc::new(RefCell::new(result)),
            })))
        }
        "join" => {
            let sep = args.first().map(|v| v.to_string()).unwrap_or_else(|| ",".to_string());
            let parts: Vec<String> = arr
                .borrow()
                .iter()
                .map(|v| match v {
                    Value::Null | Value::Undefined => String::new(),
                    other => other.to_string(),
                })
                .collect();
            Ok(Some(Value::String(parts.join(&sep))))
        }
        "reverse" => {
            arr.borrow_mut().reverse();
            Ok(Some(Value::Array(arr)))
        }
        "indexOf" => {
            let target = args.first().cloned().unwrap_or(Value::Undefined);
            let from = args.get(1).map(|v| v.to_number() as usize).unwrap_or(0);
            let borrowed = arr.borrow();
            for (i, val) in borrowed.iter().enumerate().skip(from) {
                if val == &target {
                    return Ok(Some(Value::Number(i as f64)));
                }
            }
            Ok(Some(Value::Number(-1.0)))
        }
        "lastIndexOf" => {
            let target = args.first().cloned().unwrap_or(Value::Undefined);
            let borrowed = arr.borrow();
            let from = args
                .get(1)
                .map(|v| v.to_number() as usize)
                .unwrap_or(borrowed.len().saturating_sub(1));
            for i in (0..=from.min(borrowed.len().saturating_sub(1))).rev() {
                if borrowed[i] == target {
                    return Ok(Some(Value::Number(i as f64)));
                }
            }
            Ok(Some(Value::Number(-1.0)))
        }
        "includes" => {
            let target = args.first().cloned().unwrap_or(Value::Undefined);
            let found = arr.borrow().iter().any(|v| v == &target);
            Ok(Some(Value::Boolean(found)))
        }
        "sort" => {
            let callback = args.into_iter().next();
            let mut elements = arr.borrow().clone();
            if let Some(cb) = callback {
                let mut sort_err: Option<Control> = None;
                let n = elements.len();
                'outer: for i in 1..n {
                    let mut j = i;
                    while j > 0 {
                        let result = call_fn(
                            cb.clone(),
                            Value::Undefined,
                            vec![elements[j - 1].clone(), elements[j].clone()],
                        );
                        match result {
                            Ok(cmp) => {
                                if cmp.to_number() > 0.0 {
                                    elements.swap(j - 1, j);
                                    j -= 1;
                                } else {
                                    break;
                                }
                            }
                            Err(e) => {
                                sort_err = Some(e);
                                break 'outer;
                            }
                        }
                    }
                }
                if let Some(e) = sort_err {
                    return Err(e);
                }
            } else {
                elements.sort_by_key(|a| a.to_string());
            }
            *arr.borrow_mut() = elements;
            Ok(Some(Value::Array(arr)))
        }
        "flat" => {
            let depth = args.first().map(|v| v.to_number() as usize).unwrap_or(1);
            fn flatten(vals: &[Value], depth: usize) -> Vec<Value> {
                if depth == 0 {
                    return vals.to_vec();
                }
                let mut result = Vec::new();
                for v in vals {
                    if let Value::Array(inner) = v {
                        result.extend(flatten(&inner.borrow().clone(), depth - 1));
                    } else {
                        result.push(v.clone());
                    }
                }
                result
            }
            let flattened = flatten(&arr.borrow().clone(), depth);
            Ok(Some(Value::Array(ArrayValue {
                elements: Rc::new(RefCell::new(flattened)),
            })))
        }
        "fill" => {
            let fill_val = args.first().cloned().unwrap_or(Value::Undefined);
            let mut borrowed = arr.borrow_mut();
            let len = borrowed.len();
            let start = args
                .get(1)
                .map(|v| {
                    let n = v.to_number() as i64;
                    if n < 0 { (len as i64 + n).max(0) as usize } else { (n as usize).min(len) }
                })
                .unwrap_or(0);
            let end = args
                .get(2)
                .map(|v| {
                    let n = v.to_number() as i64;
                    if n < 0 { (len as i64 + n).max(0) as usize } else { (n as usize).min(len) }
                })
                .unwrap_or(len);
            for i in start..end {
                borrowed[i] = fill_val.clone();
            }
            drop(borrowed);
            Ok(Some(Value::Array(arr)))
        }
        "at" => {
            let borrowed = arr.borrow();
            let len = borrowed.len();
            let idx = args.first().map(|v| v.to_number() as i64).unwrap_or(0);
            let actual = if idx < 0 { len as i64 + idx } else { idx } as usize;
            Ok(Some(borrowed.get(actual).cloned().unwrap_or(Value::Undefined)))
        }
        "forEach" => {
            let cb = args.into_iter().next().unwrap_or(Value::Undefined);
            let elements = arr.borrow().clone();
            for (i, val) in elements.into_iter().enumerate() {
                call_fn(cb.clone(), Value::Undefined, vec![val, Value::Number(i as f64)])?;
            }
            Ok(Some(Value::Undefined))
        }
        "map" => {
            let cb = args.into_iter().next().unwrap_or(Value::Undefined);
            let elements = arr.borrow().clone();
            let mut result = Vec::with_capacity(elements.len());
            for (i, val) in elements.into_iter().enumerate() {
                let mapped =
                    call_fn(cb.clone(), Value::Undefined, vec![val, Value::Number(i as f64)])?;
                result.push(mapped);
            }
            Ok(Some(Value::Array(ArrayValue {
                elements: Rc::new(RefCell::new(result)),
            })))
        }
        "filter" => {
            let cb = args.into_iter().next().unwrap_or(Value::Undefined);
            let elements = arr.borrow().clone();
            let mut result = Vec::new();
            for (i, val) in elements.into_iter().enumerate() {
                let pass =
                    call_fn(cb.clone(), Value::Undefined, vec![val.clone(), Value::Number(i as f64)])?;
                if pass.is_truthy() {
                    result.push(val);
                }
            }
            Ok(Some(Value::Array(ArrayValue {
                elements: Rc::new(RefCell::new(result)),
            })))
        }
        "reduce" => {
            let cb = args.first().cloned().unwrap_or(Value::Undefined);
            let elements = arr.borrow().clone();
            let (mut acc, start_idx) = if args.len() >= 2 {
                (args[1].clone(), 0)
            } else if !elements.is_empty() {
                (elements[0].clone(), 1)
            } else {
                return Err(Control::Error(String::from(
                    "Interpreter: reduce on empty array with no initial value",
                )));
            };
            for (i, val) in elements.into_iter().enumerate().skip(start_idx) {
                acc = call_fn(
                    cb.clone(),
                    Value::Undefined,
                    vec![acc, val, Value::Number(i as f64)],
                )?;
            }
            Ok(Some(acc))
        }
        "reduceRight" => {
            let cb = args.first().cloned().unwrap_or(Value::Undefined);
            let elements = arr.borrow().clone();
            let len = elements.len();
            let (mut acc, start_idx) = if args.len() >= 2 {
                (args[1].clone(), len)
            } else if len > 0 {
                (elements[len - 1].clone(), len - 1)
            } else {
                return Err(Control::Error(String::from(
                    "Interpreter: reduceRight on empty array with no initial value",
                )));
            };
            for i in (0..start_idx).rev() {
                acc = call_fn(
                    cb.clone(),
                    Value::Undefined,
                    vec![acc, elements[i].clone(), Value::Number(i as f64)],
                )?;
            }
            Ok(Some(acc))
        }
        "some" => {
            let cb = args.into_iter().next().unwrap_or(Value::Undefined);
            let elements = arr.borrow().clone();
            for (i, val) in elements.into_iter().enumerate() {
                let result =
                    call_fn(cb.clone(), Value::Undefined, vec![val, Value::Number(i as f64)])?;
                if result.is_truthy() {
                    return Ok(Some(Value::Boolean(true)));
                }
            }
            Ok(Some(Value::Boolean(false)))
        }
        "every" => {
            let cb = args.into_iter().next().unwrap_or(Value::Undefined);
            let elements = arr.borrow().clone();
            for (i, val) in elements.into_iter().enumerate() {
                let result =
                    call_fn(cb.clone(), Value::Undefined, vec![val, Value::Number(i as f64)])?;
                if !result.is_truthy() {
                    return Ok(Some(Value::Boolean(false)));
                }
            }
            Ok(Some(Value::Boolean(true)))
        }
        "find" => {
            let cb = args.into_iter().next().unwrap_or(Value::Undefined);
            let elements = arr.borrow().clone();
            for (i, val) in elements.into_iter().enumerate() {
                let result = call_fn(
                    cb.clone(),
                    Value::Undefined,
                    vec![val.clone(), Value::Number(i as f64)],
                )?;
                if result.is_truthy() {
                    return Ok(Some(val));
                }
            }
            Ok(Some(Value::Undefined))
        }
        "findIndex" => {
            let cb = args.into_iter().next().unwrap_or(Value::Undefined);
            let elements = arr.borrow().clone();
            for (i, val) in elements.into_iter().enumerate() {
                let result =
                    call_fn(cb.clone(), Value::Undefined, vec![val, Value::Number(i as f64)])?;
                if result.is_truthy() {
                    return Ok(Some(Value::Number(i as f64)));
                }
            }
            Ok(Some(Value::Number(-1.0)))
        }
        "flatMap" => {
            let cb = args.into_iter().next().unwrap_or(Value::Undefined);
            let elements = arr.borrow().clone();
            let mut result = Vec::new();
            for (i, val) in elements.into_iter().enumerate() {
                let mapped =
                    call_fn(cb.clone(), Value::Undefined, vec![val, Value::Number(i as f64)])?;
                match mapped {
                    Value::Array(inner) => result.extend(inner.borrow().clone()),
                    other => result.push(other),
                }
            }
            Ok(Some(Value::Array(ArrayValue {
                elements: Rc::new(RefCell::new(result)),
            })))
        }
        "toString" => {
            let parts: Vec<String> = arr
                .borrow()
                .iter()
                .map(|v| match v {
                    Value::Null | Value::Undefined => String::new(),
                    other => other.to_string(),
                })
                .collect();
            Ok(Some(Value::String(parts.join(","))))
        }
        "keys" => {
            let len = arr.borrow().len();
            let keys: Vec<Value> = (0..len).map(|i| Value::Number(i as f64)).collect();
            Ok(Some(Value::Array(ArrayValue {
                elements: Rc::new(RefCell::new(keys)),
            })))
        }
        "values" => Ok(Some(Value::Array(arr))),
        "entries" => {
            let entries: Vec<Value> = arr
                .borrow()
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    Value::Array(ArrayValue {
                        elements: Rc::new(RefCell::new(vec![Value::Number(i as f64), v.clone()])),
                    })
                })
                .collect();
            Ok(Some(Value::Array(ArrayValue {
                elements: Rc::new(RefCell::new(entries)),
            })))
        }
        _ => Ok(None),
    }
}

pub(super) fn register(env: &Rc<RefCell<IndexMap<String, Value>>>) {
    env.borrow_mut().insert(
        "Array".to_string(),
        make_obj(&[
            (
                "isArray",
                native(|a| Value::Boolean(matches!(a.first(), Some(Value::Array(_))))),
            ),
            (
                "from",
                native(|a| match a.first() {
                    Some(Value::Array(arr)) => Value::Array(arr.clone()),
                    Some(Value::String(s)) => {
                        let chars: Vec<Value> =
                            s.chars().map(|c| Value::String(c.to_string())).collect();
                        Value::Array(ArrayValue {
                            elements: Rc::new(RefCell::new(chars)),
                        })
                    }
                    _ => Value::Array(ArrayValue {
                        elements: Rc::new(RefCell::new(vec![])),
                    }),
                }),
            ),
            (
                "of",
                native(|a| {
                    Value::Array(ArrayValue {
                        elements: Rc::new(RefCell::new(a.to_vec())),
                    })
                }),
            ),
        ]),
    );
}
