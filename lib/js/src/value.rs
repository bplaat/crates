/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::parser::AstNode;

/// MARK: Array value
#[derive(Debug, Clone)]
pub(crate) struct ArrayValue {
    pub elements: Rc<RefCell<Vec<Value>>>,
}
impl Deref for ArrayValue {
    type Target = Rc<RefCell<Vec<Value>>>;

    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}
impl PartialEq for ArrayValue {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.elements, &other.elements)
    }
}

/// MARK: Object value
#[derive(Debug, Clone)]
pub(crate) struct ObjectValue {
    pub properties: Rc<RefCell<IndexMap<String, Value>>>,
}
impl Deref for ObjectValue {
    type Target = Rc<RefCell<IndexMap<String, Value>>>;

    fn deref(&self) -> &Self::Target {
        &self.properties
    }
}
impl PartialEq for ObjectValue {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.properties, &other.properties)
    }
}

/// MARK: Value
#[derive(Debug, Clone)]
#[allow(private_interfaces)]
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
    /// Array value
    Array(ArrayValue),
    /// Object value
    Object(ObjectValue),
    /// Function value
    Function(Rc<(Vec<String>, AstNode)>),
    /// Native function
    NativeFunction(fn(&[Value]) -> Value),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => Rc::ptr_eq(a, b),
            (Value::NativeFunction(a), Value::NativeFunction(b)) => {
                let a_ptr: usize = (*a) as usize;
                let b_ptr: usize = (*b) as usize;
                a_ptr == b_ptr
            }
            _ => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Undefined => write!(f, "undefined"),
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Array(a) => {
                let mut elements = vec![];
                for v in a.borrow().iter() {
                    elements.push(v.to_string());
                }
                write!(f, "[{}]", elements.join(", "))
            }
            Value::Object(_) => write!(f, "[object Object]"),
            Value::Function(_) | Value::NativeFunction(_) => {
                write!(f, "function() {{ [native code] }}")
            }
        }
    }
}

impl Value {
    pub(crate) fn typeof_string(&self) -> &'static str {
        match self {
            Value::Undefined => "undefined",
            Value::Null => "object",
            Value::Boolean(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) | Value::Object(_) => "object",
            Value::Function(..) | Value::NativeFunction(_) => "function",
        }
    }

    pub(crate) fn is_truthy(&self) -> bool {
        match self {
            Value::Undefined => false,
            Value::Null => false,
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0 && !n.is_nan(),
            Value::String(s) => !s.is_empty(),
            Value::Array(_) | Value::Object(_) => true,
            Value::Function(..) | Value::NativeFunction(_) => true,
        }
    }

    pub(crate) fn loose_equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Undefined, Value::Null) => true,
            (Value::Null, Value::Undefined) => true,
            (Value::Boolean(a), b) => {
                let a_num = if *a { 1.0 } else { 0.0 };
                a_num == b.to_number()
            }
            (a, Value::Boolean(b)) => {
                let b_num = if *b { 1.0 } else { 0.0 };
                a.to_number() == b_num
            }
            (Value::Number(a), b) => *a == b.to_number(),
            (a, Value::Number(b)) => a.to_number() == *b,
            (Value::String(a), b) => *a == b.to_string(),
            (a, Value::String(b)) => a.to_string() == *b,
            _ => self == other,
        }
    }

    pub(crate) fn to_number(&self) -> f64 {
        match self {
            Value::Undefined => f64::NAN,
            Value::Null => 0.0,
            Value::Boolean(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Number(n) => *n,
            Value::String(s) => s.parse::<f64>().unwrap_or(f64::NAN),
            Value::Array(a) => {
                let arr = a.borrow();
                if arr.len() == 1 {
                    arr[0].to_number()
                } else if arr.is_empty() {
                    0.0
                } else {
                    f64::NAN
                }
            }
            Value::Object(_) => f64::NAN,
            Value::Function(..) | Value::NativeFunction(_) => f64::NAN,
        }
    }
}
