/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;

use crate::value::Value;

pub(crate) fn env() -> HashMap<String, Value> {
    let mut env = HashMap::new();

    env.insert(
        "sum".to_string(),
        Value::NativeFunction(|args: Vec<Value>| {
            let mut sum = 0.0;
            for arg in args {
                if let Value::Number(n) = arg {
                    sum += n;
                } else {
                    return Err("Invalid argument to sum, expected number".to_string());
                }
            }
            Ok(Value::Number(sum))
        }),
    );

    env
}
