/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;

use crate::parser::Node;
use crate::value::Value;

pub(crate) struct Interpreter<'a> {
    env: &'a mut HashMap<String, Value>,
}

impl<'a> Interpreter<'a> {
    pub(crate) fn new(env: &'a mut HashMap<String, Value>) -> Self {
        Interpreter { env }
    }

    pub(crate) fn eval(&mut self, node: &Node) -> Result<Value, String> {
        match node {
            Node::Nodes(nodes) => {
                let mut result = Value::Undefined;
                for node in nodes {
                    result = self.eval(node)?;
                }
                Ok(result)
            }
            Node::Number(number) => Ok(Value::Number(*number)),
            Node::Variable(variable) => match self.env.get(variable) {
                Some(value) => Ok(value.clone()),
                None => Err(format!("Interpreter: variable {variable} doesn't exists")),
            },
            Node::Assign(lhs, rhs) => {
                let result = self.eval(rhs)?;
                match lhs.as_ref() {
                    Node::Variable(variable) => {
                        self.env.insert(variable.to_string(), result.clone());
                    }
                    _ => return Err(String::from("Interpreter: assign lhs is not a variable")),
                }
                Ok(result)
            }
            Node::Neg(unary) => match self.eval(unary)? {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(String::from("Interpreter: negation on non-number")),
            },
            Node::Add(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                _ => Err(String::from("Interpreter: addition on non-numbers")),
            },
            Node::Sub(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                _ => Err(String::from("Interpreter: subtraction on non-numbers")),
            },
            Node::Mul(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                _ => Err(String::from("Interpreter: multiplication on non-numbers")),
            },
            Node::Exp(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.pow(b as u32))),
                _ => Err(String::from("Interpreter: exponentiation on non-numbers")),
            },
            Node::Div(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => {
                    Ok(Value::Number(if b != 0 { a / b } else { 0 }))
                }
                _ => Err(String::from("Interpreter: division on non-numbers")),
            },
            Node::Mod(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a % b)),
                _ => Err(String::from("Interpreter: modulo on non-numbers")),
            },
            Node::BitwiseAnd(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a & b)),
                _ => Err(String::from("Interpreter: bitwise and on non-numbers")),
            },
            Node::BitwiseOr(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a | b)),
                _ => Err(String::from("Interpreter: bitwise or on non-numbers")),
            },
            Node::BitwiseXor(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a ^ b)),
                _ => Err(String::from("Interpreter: bitwise xor on non-numbers")),
            },
            Node::BitwiseNot(unary) => match self.eval(unary)? {
                Value::Number(n) => Ok(Value::Number(!n)),
                _ => Err(String::from("Interpreter: bitwise not on non-number")),
            },
            Node::LeftShift(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a << b)),
                _ => Err(String::from("Interpreter: left shift on non-numbers")),
            },
            Node::SignedRightShift(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a >> b)),
                _ => Err(String::from(
                    "Interpreter: signed right shift on non-numbers",
                )),
            },
            Node::UnsignedRightShift(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => {
                    Ok(Value::Number(((a as u64) >> (b as u64)) as i64))
                }
                _ => Err(String::from(
                    "Interpreter: unsigned right shift on non-numbers",
                )),
            },
        }
    }
}
