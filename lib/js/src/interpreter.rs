/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;

use crate::parser::{AstNode, DeclarationType};
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
        env: HashMap<String, Value>,
        return_value: Option<Value>,
    },
}

pub(crate) struct Interpreter<'a> {
    global_env: &'a mut HashMap<String, Value>,
    scopes: Vec<Scope>,
    previous_value: Option<Value>,
}

impl<'a> Interpreter<'a> {
    pub(crate) fn new(global_env: &'a mut HashMap<String, Value>) -> Self {
        Interpreter {
            global_env,
            scopes: Vec::new(),
            previous_value: None,
        }
    }

    // MARK: Eval node
    pub(crate) fn eval(&mut self, node: &AstNode) -> Result<Value, String> {
        if let Some(scope) = self.scopes.last_mut() {
            match scope {
                Scope::Loop {
                    continue_flag,
                    break_flag,
                } => {
                    if *continue_flag || *break_flag {
                        return Ok(self.previous_value.take().unwrap_or(Value::Undefined));
                    }
                }
                Scope::Switch { break_flag } => {
                    if *break_flag {
                        return Ok(self.previous_value.take().unwrap_or(Value::Undefined));
                    }
                }
                Scope::Function { return_value, .. } => {
                    if let Some(ret_val) = return_value.take() {
                        return Ok(ret_val);
                    }
                }
            }
        }

        match node {
            AstNode::Nodes(nodes) => {
                for node in nodes {
                    self.previous_value = Some(self.eval(node)?);
                }
                Ok(self.previous_value.take().unwrap_or(Value::Undefined))
            }
            AstNode::If {
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
            AstNode::Switch {
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
            AstNode::While { condition, body } => {
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
            AstNode::DoWhile { body, condition } => {
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
            AstNode::For {
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
            AstNode::Continue => match self.scopes.last_mut() {
                Some(Scope::Loop { continue_flag, .. }) => {
                    *continue_flag = true;
                    Ok(self.previous_value.take().unwrap_or(Value::Undefined))
                }
                Some(_) => Err(String::from("Interpreter: 'continue' used outside of loop")),
                None => Err(String::from("Interpreter: 'continue' used outside of loop")),
            },
            AstNode::Break => match self.scopes.last_mut() {
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
            AstNode::Return(value) => {
                let ret_value = if let Some(ret_node) = value {
                    self.eval(ret_node)?
                } else {
                    Value::Undefined
                };
                if let Some(Scope::Function { return_value, .. }) = self.scopes.last_mut() {
                    *return_value = Some(ret_value);
                    Ok(self.previous_value.take().unwrap_or(Value::Undefined))
                } else {
                    Err(String::from(
                        "Interpreter: 'return' used outside of function",
                    ))
                }
            }

            AstNode::Value(value) => Ok(value.clone()),
            AstNode::Variable(variable) => match self.get_var(variable) {
                Some(value) => Ok(value.clone()),
                None => Err(format!("Interpreter: variable {variable} doesn't exists")),
            },
            AstNode::FunctionCall(function, arguments) => {
                let func_value = self.eval(function)?;
                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.eval(arg)?);
                }
                match func_value {
                    Value::Function(arguments, body) => {
                        let mut func_env = HashMap::new();
                        for (i, arg_name) in arguments.iter().enumerate() {
                            let arg_value = arg_values.get(i).cloned().unwrap_or(Value::Undefined);
                            func_env.insert(arg_name.clone(), arg_value);
                        }
                        self.scopes.push(Scope::Function {
                            env: func_env,
                            return_value: None,
                        });
                        self.eval(&body)?;
                        if let Some(Scope::Function { return_value, .. }) = self.scopes.pop() {
                            if let Some(ret_val) = return_value {
                                Ok(ret_val)
                            } else {
                                Ok(Value::Undefined)
                            }
                        } else {
                            Err(String::from(
                                "Interpreter: function scope not found after function call",
                            ))
                        }
                    }
                    Value::NativeFunction(func) => func(arg_values),
                    _ => Err(String::from(
                        "Interpreter: trying to call a non-function value",
                    )),
                }
            }

            AstNode::Assign(declaration_type, lhs, rhs) => self.assign(*declaration_type, lhs, rhs),
            AstNode::AddAssign(lhs, rhs) => self.op_assign(lhs, rhs, |a, b| a + b, "addition"),
            AstNode::SubtractAssign(lhs, rhs) => {
                self.op_assign(lhs, rhs, |a, b| a - b, "subtraction")
            }
            AstNode::MultiplyAssign(lhs, rhs) => {
                self.op_assign(lhs, rhs, |a, b| a * b, "multiplication")
            }
            AstNode::DivideAssign(lhs, rhs) => self.op_assign(
                lhs,
                rhs,
                |a, b| if b != 0.0 { a / b } else { 0.0 },
                "division",
            ),
            AstNode::RemainderAssign(lhs, rhs) => self.op_assign(lhs, rhs, |a, b| a % b, "modulo"),
            AstNode::ExponentiationAssign(lhs, rhs) => {
                self.op_assign(lhs, rhs, |a, b| a.powf(b), "exponentiation")
            }
            AstNode::BitwiseAndAssign(lhs, rhs) => {
                self.binary_op_assign(lhs, rhs, |a, b| a & b, "bitwise and")
            }
            AstNode::BitwiseOrAssign(lhs, rhs) => {
                self.binary_op_assign(lhs, rhs, |a, b| a | b, "bitwise or")
            }
            AstNode::BitwiseXorAssign(lhs, rhs) => {
                self.binary_op_assign(lhs, rhs, |a, b| a ^ b, "bitwise xor")
            }
            AstNode::LeftShiftAssign(lhs, rhs) => {
                self.binary_op_assign(lhs, rhs, |a, b| a << b, "left shift")
            }
            AstNode::SignedRightShiftAssign(lhs, rhs) => {
                self.binary_op_assign(lhs, rhs, |a, b| a >> b, "signed right shift")
            }
            AstNode::UnsignedRightShiftAssign(lhs, rhs) => self.binary_op_assign(
                lhs,
                rhs,
                |a, b| ((a as u32) >> (b as u32)) as i32,
                "unsigned right shift",
            ),
            AstNode::LogicalOrAssign(lhs, rhs) => {
                let lhs_val = self.eval(lhs)?;
                if lhs_val.is_truthy() {
                    return Ok(lhs_val);
                }
                let rhs_val = self.eval(rhs)?;
                match &**lhs {
                    AstNode::Variable(variable) => self.set_var(variable, rhs_val.clone()),
                    _ => return Err(String::from("Interpreter: assign lhs is not a variable")),
                }
                Ok(rhs_val)
            }
            AstNode::LogicalAndAssign(lhs, rhs) => {
                let lhs_val = self.eval(lhs)?;
                if !lhs_val.is_truthy() {
                    return Ok(lhs_val);
                }
                let rhs_val = self.eval(rhs)?;
                match &**lhs {
                    AstNode::Variable(variable) => self.set_var(variable, rhs_val.clone()),
                    _ => return Err(String::from("Interpreter: assign lhs is not a variable")),
                }
                Ok(rhs_val)
            }

            AstNode::Ternary {
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

            AstNode::UnaryMinus(unary) => match self.eval(unary)? {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(String::from("Interpreter: negation on non-number")),
            },
            AstNode::UnaryLogicalNot(unary) => {
                let val = self.eval(unary)?;
                Ok(Value::Boolean(!val.is_truthy()))
            }
            AstNode::UnaryPreIncrement(unary) => match &**unary {
                AstNode::Variable(var_name) => {
                    let current_value = self.eval(unary)?;
                    match current_value {
                        Value::Number(n) => {
                            let new_value = Value::Number(n + 1.0);
                            self.set_var(var_name, new_value.clone());
                            Ok(new_value)
                        }
                        _ => Err(String::from("Interpreter: increment on non-number")),
                    }
                }
                _ => Err(String::from(
                    "Interpreter: pre-increment can only be applied to variables",
                )),
            },
            AstNode::UnaryPreDecrement(unary) => match &**unary {
                AstNode::Variable(var_name) => {
                    let current_value = self.eval(unary)?;
                    match current_value {
                        Value::Number(n) => {
                            let new_value = Value::Number(n - 1.0);
                            self.set_var(var_name, new_value.clone());
                            Ok(new_value)
                        }
                        _ => Err(String::from("Interpreter: decrement on non-number")),
                    }
                }
                _ => Err(String::from(
                    "Interpreter: pre-decrement can only be applied to variables",
                )),
            },
            AstNode::UnaryPostIncrement(unary) => match &**unary {
                AstNode::Variable(var_name) => {
                    let current_value = self.eval(unary)?;
                    match current_value {
                        Value::Number(n) => {
                            self.set_var(var_name, Value::Number(n + 1.0));
                            Ok(Value::Number(n))
                        }
                        _ => Err(String::from("Interpreter: increment on non-number")),
                    }
                }
                _ => Err(String::from(
                    "Interpreter: post-increment can only be applied to variables",
                )),
            },
            AstNode::UnaryPostDecrement(unary) => match &**unary {
                AstNode::Variable(var_name) => {
                    let current_value = self.eval(unary)?;
                    match current_value {
                        Value::Number(n) => {
                            self.set_var(var_name, Value::Number(n - 1.0));
                            Ok(Value::Number(n))
                        }
                        _ => Err(String::from("Interpreter: decrement on non-number")),
                    }
                }
                _ => Err(String::from(
                    "Interpreter: post-decrement can only be applied to variables",
                )),
            },
            AstNode::UnaryTypeof(unary) => {
                let val = self.eval(unary)?;
                Ok(Value::String(val.typeof_string().to_string()))
            }

            AstNode::Add(lhs, rhs) => self.arithmetic_op(lhs, rhs, |a, b| a + b, "addition"),
            AstNode::Subtract(lhs, rhs) => {
                self.arithmetic_op(lhs, rhs, |a, b| a - b, "subtraction")
            }
            AstNode::Multiply(lhs, rhs) => {
                self.arithmetic_op(lhs, rhs, |a, b| a * b, "multiplication")
            }
            AstNode::Divide(lhs, rhs) => self.arithmetic_op(
                lhs,
                rhs,
                |a, b| if b != 0.0 { a / b } else { 0.0 },
                "division",
            ),
            AstNode::Remainder(lhs, rhs) => self.arithmetic_op(lhs, rhs, |a, b| a % b, "modulo"),
            AstNode::Exponentiation(lhs, rhs) => {
                self.arithmetic_op(lhs, rhs, |a, b| a.powf(b), "exponentiation")
            }
            AstNode::BitwiseAnd(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a & b, "bitwise and"),
            AstNode::BitwiseOr(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a | b, "bitwise or"),
            AstNode::BitwiseXor(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a ^ b, "bitwise xor"),
            AstNode::LeftShift(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a << b, "left shift"),
            AstNode::SignedRightShift(lhs, rhs) => {
                self.binary_op(lhs, rhs, |a, b| a >> b, "signed right shift")
            }
            AstNode::UnsignedRightShift(lhs, rhs) => self.binary_op(
                lhs,
                rhs,
                |a, b| ((a as u32) >> (b as u32)) as i32,
                "unsigned right shift",
            ),
            AstNode::BitwiseNot(unary) => match self.eval(unary)? {
                Value::Number(n) => Ok(Value::Number(!(n as i32) as f64)),
                _ => Err(String::from("Interpreter: bitwise not on non-number")),
            },

            AstNode::Equals(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a == b)),
                (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a == b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a == b)),
                (Value::Undefined, Value::Undefined) => Ok(Value::Boolean(true)),
                (Value::Null, Value::Null) => Ok(Value::Boolean(true)),
                _ => Ok(Value::Boolean(false)),
            },
            AstNode::StrictEquals(lhs, rhs) => {
                let (lhs_val, rhs_val) = (self.eval(lhs)?, self.eval(rhs)?);
                Ok(Value::Boolean(lhs_val == rhs_val))
            }
            AstNode::NotEquals(lhs, rhs) => match (self.eval(lhs)?, self.eval(rhs)?) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a != b)),
                (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a != b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a != b)),
                (Value::Undefined, Value::Undefined) => Ok(Value::Boolean(false)),
                (Value::Null, Value::Null) => Ok(Value::Boolean(false)),
                _ => Ok(Value::Boolean(true)),
            },
            AstNode::StrictNotEquals(lhs, rhs) => {
                let (lhs_val, rhs_val) = (self.eval(lhs)?, self.eval(rhs)?);
                Ok(Value::Boolean(lhs_val != rhs_val))
            }
            AstNode::LessThen(lhs, rhs) => self.compare_op(lhs, rhs, |a, b| a < b, "less than"),
            AstNode::LessThenEquals(lhs, rhs) => {
                self.compare_op(lhs, rhs, |a, b| a <= b, "less than equals")
            }
            AstNode::GreaterThen(lhs, rhs) => {
                self.compare_op(lhs, rhs, |a, b| a > b, "greater than")
            }
            AstNode::GreaterThenEquals(lhs, rhs) => {
                self.compare_op(lhs, rhs, |a, b| a >= b, "greater than equals")
            }

            AstNode::LogicalAnd(lhs, rhs) => {
                let lhs_val = self.eval(lhs)?;
                if !lhs_val.is_truthy() {
                    return Ok(lhs_val);
                }
                self.eval(rhs)
            }
            AstNode::LogicalOr(lhs, rhs) => {
                let lhs_val = self.eval(lhs)?;
                if lhs_val.is_truthy() {
                    return Ok(lhs_val);
                }
                self.eval(rhs)
            }
        }
    }

    // MARK: Utils
    fn get_var(&mut self, variable: &str) -> Option<&Value> {
        for scope in self.scopes.iter_mut().rev() {
            if let Scope::Function { env, .. } = scope
                && env.contains_key(variable)
            {
                return env.get(variable);
            }
        }
        self.global_env.get(variable)
    }

    fn set_var(&mut self, variable: &str, value: Value) {
        // Try to find existing variable in function scopes
        for scope in self.scopes.iter_mut().rev() {
            if let Scope::Function { env, .. } = scope
                && env.contains_key(variable)
            {
                env.insert(variable.to_string(), value);
                return;
            }
        }

        // Try to find in global scope
        if self.global_env.contains_key(variable) {
            self.global_env.insert(variable.to_string(), value);
            return;
        }

        // Default: insert in current scope or global
        let env = if let Some(Scope::Function { env, .. }) = self.scopes.last_mut() {
            env
        } else {
            &mut self.global_env
        };
        env.insert(variable.to_string(), value);
    }

    fn assign(
        &mut self,
        _declaration_type: DeclarationType,
        lhs: &AstNode,
        rhs: &AstNode,
    ) -> Result<Value, String> {
        let result = self.eval(rhs)?;
        match lhs {
            AstNode::Variable(variable) => self.set_var(variable, result.clone()),
            _ => return Err(String::from("Interpreter: assign lhs is not a variable")),
        }
        Ok(result)
    }

    fn arithmetic_op<F>(
        &mut self,
        lhs: &AstNode,
        rhs: &AstNode,
        op: F,
        op_name: &str,
    ) -> Result<Value, String>
    where
        F: Fn(f64, f64) -> f64,
    {
        let lhs_val = self.eval(lhs)?;
        let rhs_val = self.eval(rhs)?;

        // Handle string concatenation for addition
        // FIXME: cleanup
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
        lhs: &AstNode,
        rhs: &AstNode,
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
        lhs: &AstNode,
        rhs: &AstNode,
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
        lhs: &AstNode,
        rhs: &AstNode,
        op: F,
        op_name: &str,
    ) -> Result<Value, String>
    where
        F: Fn(f64, f64) -> f64,
    {
        let lhs_val = self.eval(lhs)?;
        let rhs_val = self.eval(rhs)?;

        // Handle string concatenation for addition
        // FIXME: cleanup
        if op_name == "addition"
            && let (Value::String(a), Value::String(b)) = (&lhs_val, &rhs_val)
        {
            let result = Value::String(format!("{a}{b}"));
            match lhs {
                AstNode::Variable(variable) => self.set_var(variable, result.clone()),
                _ => return Err(String::from("Interpreter: assign lhs is not a variable")),
            }
            return Ok(result);
        }

        let result = match (lhs_val, rhs_val) {
            (Value::Number(a), Value::Number(b)) => Value::Number(op(a, b)),
            _ => return Err(format!("Interpreter: {op_name} assign on non-numbers")),
        };
        match lhs {
            AstNode::Variable(variable) => self.set_var(variable, result.clone()),
            _ => return Err(String::from("Interpreter: assign lhs is not a variable")),
        }
        Ok(result)
    }

    fn binary_op_assign<F>(
        &mut self,
        lhs: &AstNode,
        rhs: &AstNode,
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
            AstNode::Variable(variable) => self.set_var(variable, result.clone()),
            _ => return Err(String::from("Interpreter: assign lhs is not a variable")),
        }
        Ok(result)
    }
}
