/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;

use crate::parser::Node;
use crate::value::Value;

pub(crate) struct Scope {
    in_switch: bool,
    in_loop: bool,
    continue_flag: bool,
    break_flag: bool,
}

pub(crate) struct Interpreter<'a> {
    env: &'a mut HashMap<String, Value>,
    scopes: Vec<Scope>,
    previous_value: Option<Value>,
}

impl<'a> Interpreter<'a> {
    pub(crate) fn new(env: &'a mut HashMap<String, Value>) -> Self {
        Interpreter {
            env,
            scopes: Vec::new(),
            previous_value: None,
        }
    }

    // MARK: Eval node
    pub(crate) fn eval(&mut self, node: &Node) -> Result<Value, String> {
        if let Some(scope) = self.scopes.last_mut()
            && (scope.continue_flag || scope.break_flag)
        {
            return Ok(self.previous_value.take().unwrap_or(Value::Undefined));
        }
        match node {
            Node::Nodes(nodes) => {
                for node in nodes {
                    self.previous_value = Some(self.eval(node)?);
                }
                Ok(self.previous_value.take().unwrap_or(Value::Undefined))
            }
            Node::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_value = self.eval(condition)?;
                if Self::is_truthy(&cond_value) {
                    self.eval(then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.eval(else_branch)
                } else {
                    Ok(Value::Undefined)
                }
            }
            Node::Switch {
                expression,
                cases,
                default,
            } => {
                self.scopes.push(Scope {
                    in_switch: true,
                    in_loop: false,
                    continue_flag: false,
                    break_flag: false,
                });
                let expr_value = self.eval(expression)?;
                for (case_value, case_body) in cases {
                    let case_eval = self.eval(case_value)?;
                    if expr_value == case_eval {
                        let value = self.eval(case_body)?;
                        let scope = self.scopes.last().expect("Should be some");
                        if scope.break_flag {
                            self.scopes.pop();
                            return Ok(value);
                        }
                    }
                }
                if let Some(default_body) = default {
                    let value = self.eval(default_body)?;
                    self.scopes.pop();
                    return Ok(value);
                }
                self.scopes.pop();
                Ok(Value::Undefined)
            }
            Node::While { condition, body } => {
                self.scopes.push(Scope {
                    in_switch: false,
                    in_loop: true,
                    continue_flag: false,
                    break_flag: false,
                });
                let mut result = Value::Undefined;
                loop {
                    let cond_value = self.eval(condition)?;
                    if !Self::is_truthy(&cond_value) {
                        break;
                    }
                    result = self.eval(body)?;
                    let scope = self.scopes.last_mut().expect("Should be some");
                    if scope.continue_flag {
                        scope.continue_flag = false;
                    }
                    if scope.break_flag {
                        scope.break_flag = false;
                        break;
                    }
                }
                self.scopes.pop();
                Ok(result)
            }
            Node::DoWhile { body, condition } => {
                self.scopes.push(Scope {
                    in_switch: false,
                    in_loop: true,
                    continue_flag: false,
                    break_flag: false,
                });
                #[allow(unused_assignments)]
                let mut result = Value::Undefined;
                loop {
                    result = self.eval(body)?;
                    let scope = self.scopes.last_mut().expect("Should be some");
                    if scope.continue_flag {
                        scope.continue_flag = false;
                    }
                    if scope.break_flag {
                        scope.break_flag = false;
                        break;
                    }
                    let cond_value = self.eval(condition)?;
                    if !Self::is_truthy(&cond_value) {
                        break;
                    }
                }
                self.scopes.pop();
                Ok(result)
            }
            Node::For {
                init,
                condition,
                update,
                body,
            } => {
                self.scopes.push(Scope {
                    in_switch: false,
                    in_loop: true,
                    continue_flag: false,
                    break_flag: false,
                });
                if let Some(init_node) = init {
                    self.eval(init_node)?;
                }
                let mut result = Value::Undefined;
                loop {
                    if let Some(cond_node) = condition {
                        let cond_value = self.eval(cond_node)?;
                        if !Self::is_truthy(&cond_value) {
                            break;
                        }
                    }
                    result = self.eval(body)?;
                    let scope = self.scopes.last_mut().expect("Should be some");
                    if scope.continue_flag {
                        scope.continue_flag = false;
                    }
                    if scope.break_flag {
                        scope.break_flag = false;
                        break;
                    }
                    if let Some(update_node) = update {
                        self.eval(update_node)?;
                    }
                }
                self.scopes.pop();
                Ok(result)
            }
            Node::Continue => match self.scopes.last_mut() {
                Some(scope) => {
                    if !scope.in_loop {
                        return Err(String::from("Interpreter: 'continue' used outside of loop"));
                    }
                    scope.continue_flag = true;
                    Ok(self.previous_value.take().unwrap_or(Value::Undefined))
                }
                None => Err(String::from("Interpreter: 'continue' used outside of loop")),
            },
            Node::Break => match self.scopes.last_mut() {
                Some(scope) => {
                    if !scope.in_loop && !scope.in_switch {
                        return Err(String::from(
                            "Interpreter: 'break' used outside of loop or switch",
                        ));
                    }
                    scope.break_flag = true;
                    Ok(self.previous_value.take().unwrap_or(Value::Undefined))
                }
                None => Err(String::from(
                    "Interpreter: 'break' used outside of loop or switch",
                )),
            },

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
            Node::DivideAssign(lhs, rhs) => self.op_assign(
                lhs,
                rhs,
                |a, b| if b != 0.0 { a / b } else { 0.0 },
                "division",
            ),
            Node::RemainderAssign(lhs, rhs) => self.op_assign(lhs, rhs, |a, b| a % b, "modulo"),
            Node::ExponentiationAssign(lhs, rhs) => {
                self.op_assign(lhs, rhs, |a, b| a.powf(b), "exponentiation")
            }
            Node::BitwiseAndAssign(lhs, rhs) => {
                self.binary_op_assign(lhs, rhs, |a, b| a & b, "bitwise and")
            }
            Node::BitwiseOrAssign(lhs, rhs) => {
                self.binary_op_assign(lhs, rhs, |a, b| a | b, "bitwise or")
            }
            Node::BitwiseXorAssign(lhs, rhs) => {
                self.binary_op_assign(lhs, rhs, |a, b| a ^ b, "bitwise xor")
            }
            Node::LeftShiftAssign(lhs, rhs) => {
                self.binary_op_assign(lhs, rhs, |a, b| a << b, "left shift")
            }
            Node::SignedRightShiftAssign(lhs, rhs) => {
                self.binary_op_assign(lhs, rhs, |a, b| a >> b, "signed right shift")
            }
            Node::UnsignedRightShiftAssign(lhs, rhs) => self.binary_op_assign(
                lhs,
                rhs,
                |a, b| ((a as u32) >> (b as u32)) as i32,
                "unsigned right shift",
            ),

            Node::Ternary {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_value = self.eval(condition)?;
                if Self::is_truthy(&cond_value) {
                    self.eval(then_branch)
                } else {
                    self.eval(else_branch)
                }
            }

            Node::UnaryMinus(unary) => match self.eval(unary)? {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(String::from("Interpreter: negation on non-number")),
            },
            Node::UnaryLogicalNot(unary) => {
                let val = self.eval(unary)?;
                Ok(Value::Boolean(!Self::is_truthy(&val)))
            }
            Node::UnaryPreIncrement(unary) => match &**unary {
                Node::Variable(var_name) => {
                    let current_value = self.eval(unary)?;
                    match current_value {
                        Value::Number(n) => {
                            let new_value = Value::Number(n + 1.0);
                            self.env.insert(var_name.to_string(), new_value.clone());
                            Ok(new_value)
                        }
                        _ => Err(String::from("Interpreter: increment on non-number")),
                    }
                }
                _ => Err(String::from(
                    "Interpreter: pre-increment can only be applied to variables",
                )),
            },
            Node::UnaryPreDecrement(unary) => match &**unary {
                Node::Variable(var_name) => {
                    let current_value = self.eval(unary)?;
                    match current_value {
                        Value::Number(n) => {
                            let new_value = Value::Number(n - 1.0);
                            self.env.insert(var_name.to_string(), new_value.clone());
                            Ok(new_value)
                        }
                        _ => Err(String::from("Interpreter: decrement on non-number")),
                    }
                }
                _ => Err(String::from(
                    "Interpreter: pre-decrement can only be applied to variables",
                )),
            },
            Node::UnaryPostIncrement(unary) => match &**unary {
                Node::Variable(var_name) => {
                    let current_value = self.eval(unary)?;
                    match current_value {
                        Value::Number(n) => {
                            let new_value = Value::Number(n + 1.0);
                            self.env.insert(var_name.to_string(), new_value);
                            Ok(Value::Number(n))
                        }
                        _ => Err(String::from("Interpreter: increment on non-number")),
                    }
                }
                _ => Err(String::from(
                    "Interpreter: post-increment can only be applied to variables",
                )),
            },
            Node::UnaryPostDecrement(unary) => match &**unary {
                Node::Variable(var_name) => {
                    let current_value = self.eval(unary)?;
                    match current_value {
                        Value::Number(n) => {
                            let new_value = Value::Number(n - 1.0);
                            self.env.insert(var_name.to_string(), new_value);
                            Ok(Value::Number(n))
                        }
                        _ => Err(String::from("Interpreter: decrement on non-number")),
                    }
                }
                _ => Err(String::from(
                    "Interpreter: post-decrement can only be applied to variables",
                )),
            },
            Node::UnaryTypeof(unary) => {
                let val = self.eval(unary)?;
                let type_str = match val {
                    Value::Undefined => "undefined",
                    Value::Null => "object",
                    Value::Boolean(_) => "boolean",
                    Value::Number(_) => "number",
                    Value::String(_) => "string",
                };
                Ok(Value::String(type_str.to_string()))
            }

            Node::Add(lhs, rhs) => self.arithmetic_op(lhs, rhs, |a, b| a + b, "addition"),
            Node::Subtract(lhs, rhs) => self.arithmetic_op(lhs, rhs, |a, b| a - b, "subtraction"),
            Node::Multiply(lhs, rhs) => {
                self.arithmetic_op(lhs, rhs, |a, b| a * b, "multiplication")
            }
            Node::Divide(lhs, rhs) => self.arithmetic_op(
                lhs,
                rhs,
                |a, b| if b != 0.0 { a / b } else { 0.0 },
                "division",
            ),
            Node::Remainder(lhs, rhs) => self.arithmetic_op(lhs, rhs, |a, b| a % b, "modulo"),
            Node::Exponentiation(lhs, rhs) => {
                self.arithmetic_op(lhs, rhs, |a, b| a.powf(b), "exponentiation")
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
                |a, b| ((a as u32) >> (b as u32)) as i32,
                "unsigned right shift",
            ),
            Node::BitwiseNot(unary) => match self.eval(unary)? {
                Value::Number(n) => Ok(Value::Number(!(n as i32) as f64)),
                _ => Err(String::from("Interpreter: bitwise not on non-number")),
            },

            Node::Equals(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a == b)),
                (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a == b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a == b)),
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
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a != b)),
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

    // MARK: Utils
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

    fn arithmetic_op<F>(
        &mut self,
        lhs: &Node,
        rhs: &Node,
        op: F,
        op_name: &str,
    ) -> Result<Value, String>
    where
        F: Fn(f64, f64) -> f64,
    {
        let lhs_val = self.eval(lhs)?;
        let rhs_val = self.eval(rhs)?;

        // Handle string concatenation for addition
        if op_name == "addition"
            && let (Value::String(a), Value::String(b)) = (&lhs_val, &rhs_val)
        {
            return Ok(Value::String(format!("{a}{b}")));
        }

        match (lhs_val, rhs_val) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(op(a, b))),
            _ => Err(format!("Interpreter: {op_name} on non-numbers")),
        }
    }

    fn binary_op<F>(
        &mut self,
        lhs: &Node,
        rhs: &Node,
        op: F,
        op_name: &str,
    ) -> Result<Value, String>
    where
        F: Fn(i32, i32) -> i32,
    {
        match (self.eval(lhs)?, self.eval(rhs)?) {
            (Value::Number(a), Value::Number(b)) => {
                Ok(Value::Number(op(a as i32, b as i32) as f64))
            }
            _ => Err(format!("Interpreter: {op_name} on non-numbers")),
        }
    }

    fn compare_op<F>(
        &mut self,
        lhs: &Node,
        rhs: &Node,
        op: F,
        op_name: &str,
    ) -> Result<Value, String>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (self.eval(lhs)?, self.eval(rhs)?) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(op(a, b))),
            _ => Err(format!("Interpreter: {op_name} on non-numbers")),
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
        F: Fn(f64, f64) -> f64,
    {
        let lhs_val = self.eval(lhs)?;
        let rhs_val = self.eval(rhs)?;

        // Handle string concatenation for addition
        if op_name == "addition"
            && let (Value::String(a), Value::String(b)) = (&lhs_val, &rhs_val)
        {
            let result = Value::String(format!("{a}{b}"));
            match lhs {
                Node::Variable(variable) => {
                    self.env.insert(variable.to_string(), result.clone());
                }
                _ => return Err(String::from("Interpreter: assign lhs is not a variable")),
            }
            return Ok(result);
        }

        let result = match (lhs_val, rhs_val) {
            (Value::Number(a), Value::Number(b)) => Value::Number(op(a, b)),
            _ => return Err(format!("Interpreter: {op_name} assign on non-numbers")),
        };
        match lhs {
            Node::Variable(variable) => {
                self.env.insert(variable.to_string(), result.clone());
            }
            _ => return Err(String::from("Interpreter: assign lhs is not a variable")),
        }
        Ok(result)
    }

    fn binary_op_assign<F>(
        &mut self,
        lhs: &Node,
        rhs: &Node,
        op: F,
        op_name: &str,
    ) -> Result<Value, String>
    where
        F: Fn(i32, i32) -> i32,
    {
        let lhs_val = self.eval(lhs)?;
        let rhs_val = self.eval(rhs)?;
        let result = match (lhs_val, rhs_val) {
            (Value::Number(a), Value::Number(b)) => Value::Number(op(a as i32, b as i32) as f64),
            _ => return Err(format!("Interpreter: {op_name} assign on non-numbers")),
        };
        match lhs {
            Node::Variable(variable) => {
                self.env.insert(variable.to_string(), result.clone());
            }
            _ => return Err(String::from("Interpreter: assign lhs is not a variable")),
        }
        Ok(result)
    }

    fn is_truthy(value: &Value) -> bool {
        match value {
            Value::Undefined => false,
            Value::Null => false,
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
        }
    }
}
