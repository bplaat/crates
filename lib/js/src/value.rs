/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(unpredictable_function_pointer_comparisons)]

/// Value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Undefined value
    Undefined,
    /// Null value
    Null,
    /// Boolean value
    Boolean(bool),
    /// Number value
    Number(f64),
    /// String value
    String(String),
    /// Native function
    NativeFunction(fn(Vec<Value>) -> Result<Value, String>),
}

impl Value {
    pub(crate) fn typeof_string(&self) -> &'static str {
        match self {
            Value::Undefined => "undefined",
            Value::Null => "object",
            Value::Boolean(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::NativeFunction(_) => "function",
        }
    }
}
