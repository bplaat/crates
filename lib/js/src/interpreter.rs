/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;

use crate::parser::Node;
use crate::value::Value;

enum Scope {
    Switch {
        break_flag: bool,
    },
    Loop {
        continue_flag: bool,
        break_flag: bool,
    },
    Function {
        return_value: Option<Value>,
    },
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
        if let Some(scope) = self.scopes.last_mut() {
            match scope {
                Scope::Loop {
                    continue_flag,
                    break_flag,
                } if *continue_flag || *break_flag => {
                    return Ok(self.previous_value.take().unwrap_or(Value::Undefined));
                }
                Scope::Switch { break_flag } if *break_flag => {
                    return Ok(self.previous_value.take().unwrap_or(Value::Undefined));
                }
                Scope::Function { return_value } => {
                    if let Some(ret_val) = return_value.take() {
                        return Ok(ret_val);
                    }
                }
                _ => unimplemented!(),
            }
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
                if cond_value.is_truthy() {
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
                self.scopes.push(Scope::Switch { break_flag: false });
                let expr_value = self.eval(expression)?;
                for (case_value, case_body) in cases {
                    let case_eval = self.eval(case_value)?;
                    if expr_value == case_eval {
                        let value = self.eval(case_body)?;
                        if let Scope::Switch { break_flag } =
                            self.scopes.last().expect("Should be some")
                            && *break_flag
                        {
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
                self.scopes.push(Scope::Loop {
                    continue_flag: false,
                    break_flag: false,
                });
                let mut result = Value::Undefined;
                loop {
                    let cond_value = self.eval(condition)?;
                    if !cond_value.is_truthy() {
                        break;
                    }
                    result = self.eval(body)?;
                    if let Scope::Loop {
                        continue_flag,
                        break_flag,
                    } = self.scopes.last_mut().expect("Should be some")
                    {
                        if *continue_flag {
                            *continue_flag = false;
                        }
                        if *break_flag {
                            *break_flag = false;
                            break;
                        }
                    }
                }
                self.scopes.pop();
                Ok(result)
            }
            Node::DoWhile { body, condition } => {
                self.scopes.push(Scope::Loop {
                    continue_flag: false,
                    break_flag: false,
                });
                #[allow(unused_assignments)]
                let mut result = Value::Undefined;
                loop {
                    result = self.eval(body)?;
                    if let Scope::Loop {
                        continue_flag,
                        break_flag,
                    } = self.scopes.last_mut().expect("Should be some")
                    {
                        if *continue_flag {
                            *continue_flag = false;
                        }
                        if *break_flag {
                            *break_flag = false;
                            break;
                        }
                    }
                    let cond_value = self.eval(condition)?;
                    if !cond_value.is_truthy() {
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
                self.scopes.push(Scope::Loop {
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
                        if !cond_value.is_truthy() {
                            break;
                        }
                    }
                    result = self.eval(body)?;
                    if let Scope::Loop {
                        continue_flag,
                        break_flag,
                    } = self.scopes.last_mut().expect("Should be some")
                    {
                        if *continue_flag {
                            *continue_flag = false;
                        }
                        if *break_flag {
                            *break_flag = false;
                            break;
                        }
                    }
                    if let Some(update_node) = update {
                        self.eval(update_node)?;
                    }
                }
                self.scopes.pop();
                Ok(result)
            }
            Node::Continue => match self.scopes.last_mut() {
                Some(Scope::Loop { continue_flag, .. }) => {
                    *continue_flag = true;
                    Ok(self.previous_value.take().unwrap_or(Value::Undefined))
                }
                Some(_) => Err(String::from("Interpreter: 'continue' used outside of loop")),
                None => Err(String::from("Interpreter: 'continue' used outside of loop")),
            },
            Node::Break => match self.scopes.last_mut() {
                Some(Scope::Loop { break_flag, .. }) | Some(Scope::Switch { break_flag }) => {
                    *break_flag = true;
                    Ok(self.previous_value.take().unwrap_or(Value::Undefined))
                }
                Some(_) => Err(String::from(
                    "Interpreter: 'break' used outside of loop or switch",
                )),
                None => Err(String::from(
                    "Interpreter: 'break' used outside of loop or switch",
                )),
            },
            Node::FunctionDefinition {
                name,
                arguments,
                body,
            } => {
                let func_value = Value::Function {
                    arguments: arguments.clone(),
                    body: body.clone(),
                };
                self.env.insert(name.clone(), func_value.clone());
                Ok(func_value)
            }
            Node::Return(value) => {
                let ret_value = if let Some(ret_node) = value {
                    self.eval(ret_node)?
                } else {
                    Value::Undefined
                };
                if let Some(Scope::Function { return_value }) = self.scopes.last_mut() {
                    *return_value = Some(ret_value);
                    Ok(self.previous_value.take().unwrap_or(Value::Undefined))
                } else {
                    Err(String::from(
                        "Interpreter: 'return' used outside of function",
                    ))
                }
            }

            Node::Value(value) => Ok(value.clone()),
            Node::Variable(variable) => match self.env.get(variable) {
                Some(value) => Ok(value.clone()),
                None => Err(format!("Interpreter: variable {variable} doesn't exists")),
            },
            Node::FunctionCall(function, arguments) => {
                let func_value = self.eval(function)?;
                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.eval(arg)?);
                }
                match func_value {
                    Value::Function { arguments, body } => {
                        let mut func_env = self.env.clone();
                        for (i, arg_name) in arguments.iter().enumerate() {
                            let arg_value = arg_values.get(i).cloned().unwrap_or(Value::Undefined);
                            func_env.insert(arg_name.clone(), arg_value);
                        }
                        let mut func_env = self.env.clone();
                        for (i, arg_name) in arguments.iter().enumerate() {
                            func_env.insert(arg_name.clone(), arg_values[i].clone());
                        }
                        let mut func_interpreter = Interpreter::new(&mut func_env);
                        func_interpreter
                            .scopes
                            .push(Scope::Function { return_value: None });
                        let result = func_interpreter.eval(&body)?;
                        Ok(result)
                    }
                    Value::NativeFunction(func) => func(arg_values),
                    _ => Err(String::from(
                        "Interpreter: trying to call a non-function value",
                    )),
                }
            }

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
                if cond_value.is_truthy() {
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
                Ok(Value::Boolean(!val.is_truthy()))
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
                Ok(Value::String(val.typeof_string().to_string()))
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
                if !lhs_val.is_truthy() {
                    return Ok(lhs_val);
                }
                self.eval(rhs)
            }
            Node::LogicalOr(lhs, rhs) => {
                let lhs_val = self.eval(lhs)?;
                if lhs_val.is_truthy() {
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
}
