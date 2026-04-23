/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use js::{Context, Value};

pub(crate) fn assert_js(expected: Value, script: &str) {
    let mut context = Context::new();
    let result = context.eval(script);
    match result {
        Ok(actual) if actual == expected => {
            assert_eq!(expected, actual);
        }
        Ok(_) | Err(_) => {
            let mut context = Context::new();
            context.set_verbose(true);
            match context.eval(script) {
                Ok(actual) => assert_eq!(expected, actual),
                Err(error) => panic!("failed to evaluate script: {error:?}"),
            }
        }
    }
}
