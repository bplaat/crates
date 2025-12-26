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

            Node::Value(value) => Ok(value.clone()),
            Node::Variable(variable) => match self.env.get(variable) {
                Some(value) => Ok(value.clone()),
                None => Err(format!("Interpreter: variable {variable} doesn't exists")),
            },

            Node::Assign(lhs, rhs) => self.assign(lhs, rhs),
            Node::AddAssign(lhs, rhs) => self.op_assign(lhs, rhs, |a, b| a + b, "addition"),
            Node::SubtractAssign(lhs, rhs) => self.op_assign(lhs, rhs, |a, b| a - b, "subtraction"),
            Node::MultiplyAssign(lhs, rhs) => {
                self.op_assign(lhs, rhs, |a, b| a * b, "multiplication")
            }
            Node::DivideAssign(lhs, rhs) => {
                self.op_assign(lhs, rhs, |a, b| if b != 0 { a / b } else { 0 }, "division")
            }
            Node::RemainderAssign(lhs, rhs) => self.op_assign(lhs, rhs, |a, b| a % b, "modulo"),
            Node::ExponentiationAssign(lhs, rhs) => {
                self.op_assign(lhs, rhs, |a, b| a.pow(b as u32), "exponentiation")
            }
            Node::BitwiseAndAssign(lhs, rhs) => {
                self.op_assign(lhs, rhs, |a, b| a & b, "bitwise and")
            }
            Node::BitwiseOrAssign(lhs, rhs) => self.op_assign(lhs, rhs, |a, b| a | b, "bitwise or"),
            Node::BitwiseXorAssign(lhs, rhs) => {
                self.op_assign(lhs, rhs, |a, b| a ^ b, "bitwise xor")
            }
            Node::LeftShiftAssign(lhs, rhs) => {
                self.op_assign(lhs, rhs, |a, b| a << b, "left shift")
            }
            Node::SignedRightShiftAssign(lhs, rhs) => {
                self.op_assign(lhs, rhs, |a, b| a >> b, "signed right shift")
            }
            Node::UnsignedRightShiftAssign(lhs, rhs) => self.op_assign(
                lhs,
                rhs,
                |a, b| ((a as u64) >> (b as u64)) as i64,
                "unsigned right shift",
            ),

            Node::UnaryMinus(unary) => match self.eval(unary)? {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(String::from("Interpreter: negation on non-number")),
            },
            Node::UnaryLogicalNot(unary) => {
                let val = self.eval(unary)?;
                Ok(Value::Boolean(!Self::is_truthy(&val)))
            }

            Node::Add(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a + b, "addition"),
            Node::Subtract(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a - b, "subtraction"),
            Node::Multiply(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a * b, "multiplication"),
            Node::Divide(lhs, rhs) => {
                self.binary_op(lhs, rhs, |a, b| if b != 0 { a / b } else { 0 }, "division")
            }
            Node::Remainder(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a % b, "modulo"),
            Node::Exponentiation(lhs, rhs) => {
                self.binary_op(lhs, rhs, |a, b| a.pow(b as u32), "exponentiation")
            }
            Node::BitwiseAnd(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a & b, "bitwise and"),
            Node::BitwiseOr(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a | b, "bitwise or"),
            Node::BitwiseXor(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a ^ b, "bitwise xor"),
            Node::LeftShift(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a << b, "left shift"),
            Node::SignedRightShift(lhs, rhs) => {
                self.binary_op(lhs, rhs, |a, b| a >> b, "signed right shift")
            }
            Node::UnsignedRightShift(lhs, rhs) => self.binary_op(
                lhs,
                rhs,
                |a, b| ((a as u64) >> (b as u64)) as i64,
                "unsigned right shift",
            ),
            Node::BitwiseNot(unary) => match self.eval(unary)? {
                Value::Number(n) => Ok(Value::Number(!n)),
                _ => Err(String::from("Interpreter: bitwise not on non-number")),
            },

            Node::Equals(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a == b)),
                (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a == b)),
                (Value::Undefined, Value::Undefined) => Ok(Value::Boolean(true)),
                (Value::Null, Value::Null) => Ok(Value::Boolean(true)),
                _ => Ok(Value::Boolean(false)),
            },
            Node::StrictEquals(lhs, rhs) => {
                let (lhs_val, rhs_val) = (self.eval(lhs)?, self.eval(rhs)?);
                Ok(Value::Boolean(lhs_val == rhs_val))
            }
            Node::NotEquals(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a != b)),
                (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a != b)),
                (Value::Undefined, Value::Undefined) => Ok(Value::Boolean(false)),
                (Value::Null, Value::Null) => Ok(Value::Boolean(false)),
                _ => Ok(Value::Boolean(true)),
            },
            Node::StrictNotEquals(lhs, rhs) => {
                let (lhs_val, rhs_val) = (self.eval(lhs)?, self.eval(rhs)?);
                Ok(Value::Boolean(lhs_val != rhs_val))
            }
            Node::LessThen(lhs, rhs) => self.compare_op(lhs, rhs, |a, b| a < b, "less than"),
            Node::LessThenEquals(lhs, rhs) => {
                self.compare_op(lhs, rhs, |a, b| a <= b, "less than equals")
            }
            Node::GreaterThen(lhs, rhs) => self.compare_op(lhs, rhs, |a, b| a > b, "greater than"),
            Node::GreaterThenEquals(lhs, rhs) => {
                self.compare_op(lhs, rhs, |a, b| a >= b, "greater than equals")
            }

            Node::LogicalAnd(lhs, rhs) => {
                let lhs_val = self.eval(lhs)?;
                if !Self::is_truthy(&lhs_val) {
                    return Ok(lhs_val);
                }
                self.eval(rhs)
            }
            Node::LogicalOr(lhs, rhs) => {
                let lhs_val = self.eval(lhs)?;
                if Self::is_truthy(&lhs_val) {
                    return Ok(lhs_val);
                }
                self.eval(rhs)
            }
        }
    }

    fn assign(&mut self, lhs: &Node, rhs: &Node) -> Result<Value, String> {
        let result = self.eval(rhs)?;
        match lhs {
            Node::Variable(variable) => {
                self.env.insert(variable.to_string(), result.clone());
            }
            _ => return Err(String::from("Interpreter: assign lhs is not a variable")),
        }
        Ok(result)
    }

    fn binary_op<F>(
        &mut self,
        lhs: &Node,
        rhs: &Node,
        op: F,
        op_name: &str,
    ) -> Result<Value, String>
    where
        F: Fn(i64, i64) -> i64,
    {
        match (self.eval(lhs)?, self.eval(rhs)?) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(op(a, b))),
            _ => Err(format!("Interpreter: {} on non-numbers", op_name)),
        }
    }

    fn op_assign<F>(
        &mut self,
        lhs: &Node,
        rhs: &Node,
        op: F,
        op_name: &str,
    ) -> Result<Value, String>
    where
        F: Fn(i64, i64) -> i64,
    {
        let lhs_val = self.eval(lhs)?;
        let rhs_val = self.eval(rhs)?;
        let result = match (lhs_val, rhs_val) {
            (Value::Number(a), Value::Number(b)) => Value::Number(op(a, b)),
            _ => return Err(format!("Interpreter: {} assign on non-numbers", op_name)),
        };
        match lhs {
            Node::Variable(variable) => {
                self.env.insert(variable.to_string(), result.clone());
            }
            _ => return Err(String::from("Interpreter: assign lhs is not a variable")),
        }
        Ok(result)
    }

    fn compare_op<F>(
        &mut self,
        lhs: &Node,
        rhs: &Node,
        op: F,
        op_name: &str,
    ) -> Result<Value, String>
    where
        F: Fn(i64, i64) -> bool,
    {
        match (self.eval(lhs)?, self.eval(rhs)?) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(op(a, b))),
            _ => Err(format!("Interpreter: {} on non-numbers", op_name)),
        }
    }

    fn is_truthy(value: &Value) -> bool {
        match value {
            Value::Undefined => false,
            Value::Null => false,
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0,
        }
    }
}
