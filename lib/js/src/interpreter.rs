/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::parser::{AstNode, DeclarationType, ObjectProperty, SwitchClause};
use crate::value::{ArrayValue, ClosureEnv, ObjectValue, Value};

type ScopeEnv = Rc<RefCell<IndexMap<String, Value>>>;

enum Scope {
    Function(ScopeEnv),
    Block(ScopeEnv),
}

impl Scope {
    fn env_rc(&self) -> &ScopeEnv {
        match self {
            Scope::Function(e) | Scope::Block(e) => e,
        }
    }

    fn is_function(&self) -> bool {
        matches!(self, Scope::Function(_))
    }
}

fn new_scope_env() -> ScopeEnv {
    Rc::new(RefCell::new(IndexMap::new()))
}

pub(crate) enum Control {
    Error(String),
    Throw(Value),
    Break(Option<String>),
    Continue(Option<String>),
    Return(Value),
}

pub(crate) struct Interpreter {
    global_env: Rc<RefCell<IndexMap<String, Value>>>,
    scopes: Vec<Scope>,
    previous_value: Value,
}

impl Interpreter {
    pub(crate) fn new(global_env: Rc<RefCell<IndexMap<String, Value>>>) -> Self {
        Interpreter {
            global_env,
            scopes: Vec::new(),
            previous_value: Value::Undefined,
        }
    }

    // MARK: Eval
    pub(crate) fn eval(&mut self, node: &AstNode) -> Result<Value, String> {
        match self.eval_node(node) {
            Ok(val) => Ok(val),
            Err(Control::Error(err)) => Err(err),
            Err(Control::Break(_)) => Err(String::from(
                "Interpreter: 'break' used outside of loop or switch",
            )),
            Err(Control::Continue(_)) => {
                Err(String::from("Interpreter: 'continue' used outside of loop"))
            }
            Err(Control::Throw(value)) => Err(format!("Uncaught exception: {value}")),
            Err(Control::Return(_)) => Err(String::from(
                "Interpreter: 'return' used outside of function",
            )),
        }
    }

    fn eval_node(&mut self, node: &AstNode) -> Result<Value, Control> {
        match node {
            AstNode::Block { label, nodes } => {
                self.scopes.push(Scope::Block(new_scope_env()));
                // Hoist var declarations (recursively across blocks) and function declarations
                // (direct block level only) into the nearest enclosing function scope.
                let func_env = self.find_function_scope_env();
                for n in nodes {
                    self.hoist_vars_from_node(n, &func_env);
                }
                self.hoist_funcs_from_block(nodes, &func_env);
                for node in nodes {
                    self.previous_value = match self.eval_node(node) {
                        Ok(val) => val,
                        Err(Control::Break(break_label))
                            if label.is_some() && break_label == *label =>
                        {
                            self.scopes.pop();
                            return Ok(self.previous_value.clone());
                        }
                        Err(control) => {
                            self.scopes.pop();
                            return Err(control);
                        }
                    };
                }
                self.scopes.pop();
                Ok(self.previous_value.clone())
            }
            AstNode::If {
                label,
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_value = self.eval_node(condition)?;
                if cond_value.is_truthy() {
                    match self.eval_node(then_branch) {
                        Err(Control::Break(break_label))
                            if label.is_some() && break_label == *label =>
                        {
                            Ok(self.previous_value.clone())
                        }
                        result => result,
                    }
                } else if let Some(else_branch) = else_branch {
                    match self.eval_node(else_branch) {
                        Err(Control::Break(break_label))
                            if label.is_some() && break_label == *label =>
                        {
                            Ok(self.previous_value.clone())
                        }
                        result => result,
                    }
                } else {
                    Ok(Value::Undefined)
                }
            }
            AstNode::Try {
                try_block,
                catch_param,
                catch_block,
                finally_block,
            } => {
                let completion = match self.eval_node(try_block) {
                    Err(Control::Throw(value)) => {
                        if let (Some(catch_param), Some(catch_block)) =
                            (catch_param.as_ref(), catch_block.as_deref())
                        {
                            let catch_env = Rc::new(RefCell::new({
                                let mut m = IndexMap::new();
                                m.insert(catch_param.clone(), value);
                                m
                            }));
                            self.scopes.push(Scope::Block(catch_env));
                            let result = self.eval_node(catch_block);
                            self.scopes.pop();
                            result
                        } else {
                            Err(Control::Throw(value))
                        }
                    }
                    result => result,
                };

                if let Some(finally_block) = finally_block {
                    match self.eval_node(finally_block) {
                        Ok(_) => {}
                        Err(control) => return Err(control),
                    }
                }

                completion
            }
            AstNode::Switch {
                label,
                expression,
                clauses,
            } => {
                let expr_value = self.eval_node(expression)?;
                let mut last_value = Value::Undefined;

                // Phase 1: scan for the first strictly-matching case and the default position.
                // Case expressions are evaluated lazily - stop evaluating once a match is found.
                let mut start_idx: Option<usize> = None;
                let mut default_idx: Option<usize> = None;
                for (i, clause) in clauses.iter().enumerate() {
                    match clause {
                        SwitchClause::Case(case_expr, _) => {
                            if start_idx.is_none() {
                                let case_eval = self.eval_node(case_expr)?;
                                if expr_value.js_strict_equals(&case_eval) {
                                    start_idx = Some(i);
                                }
                            }
                        }
                        SwitchClause::Default(_) => {
                            if default_idx.is_none() {
                                default_idx = Some(i);
                            }
                        }
                    }
                }

                // If no case matched, fall back to the default clause.
                let start = start_idx.or(default_idx);

                // Phase 2: execute from the matching clause forward with fall-through.
                if let Some(start) = start {
                    for clause in &clauses[start..] {
                        let body = match clause {
                            SwitchClause::Case(_, body) | SwitchClause::Default(body) => body,
                        };
                        last_value = match self.eval_node(body) {
                            Err(Control::Break(break_label)) if break_label == *label => {
                                return Ok(self.previous_value.clone());
                            }
                            result => result,
                        }?;
                    }
                }

                Ok(last_value)
            }
            AstNode::While {
                label,
                condition,
                body,
            } => {
                let mut last_value = Value::Undefined;
                loop {
                    let cond_value = self.eval_node(condition)?;
                    if !cond_value.is_truthy() {
                        break;
                    }

                    last_value = match self.eval_node(body) {
                        Err(Control::Break(break_label)) if break_label == *label => {
                            return Ok(self.previous_value.clone());
                        }
                        Err(Control::Continue(continue_label)) if continue_label == *label => {
                            continue;
                        }
                        result => result,
                    }?;
                }
                Ok(last_value)
            }
            AstNode::DoWhile {
                label,
                body,
                condition,
            } => {
                let mut last_value = Value::Undefined;
                loop {
                    last_value = match self.eval_node(body) {
                        Err(Control::Break(break_label)) if break_label == *label => {
                            return Ok(self.previous_value.clone());
                        }
                        Err(Control::Continue(continue_label)) if continue_label == *label => {
                            if !self.eval_node(condition)?.is_truthy() {
                                break;
                            }
                            continue;
                        }
                        result => result,
                    }?;

                    if !self.eval_node(condition)?.is_truthy() {
                        break;
                    }
                }
                Ok(last_value)
            }
            AstNode::For {
                label,
                init,
                condition,
                update,
                body,
            } => {
                if let Some(init_node) = init {
                    self.eval_node(init_node)?;
                }

                let mut last_value = Value::Undefined;
                loop {
                    if let Some(cond_node) = condition {
                        let cond_value = self.eval_node(cond_node)?;
                        if !cond_value.is_truthy() {
                            break;
                        }
                    }

                    last_value = match self.eval_node(body) {
                        Err(Control::Break(break_label)) if break_label == *label => {
                            return Ok(self.previous_value.clone());
                        }
                        Err(Control::Continue(continue_label)) if continue_label == *label => {
                            if let Some(update_node) = update {
                                self.eval_node(update_node)?;
                            }
                            continue;
                        }
                        result => result,
                    }?;

                    if let Some(update_node) = update {
                        self.eval_node(update_node)?;
                    }
                }
                Ok(last_value)
            }
            AstNode::ForIn {
                label,
                variable,
                declaration_type,
                iterable,
                body,
            } => {
                let iterable_value = self.eval_node(iterable)?;

                // Get the keys to iterate over
                let keys = match &iterable_value {
                    Value::Object(obj) => obj.borrow().keys().cloned().collect::<Vec<_>>(),
                    Value::Array(arr) => {
                        let arr_borrowed = arr.borrow();
                        let mut indices = Vec::new();
                        for (idx, val) in arr_borrowed.iter().enumerate() {
                            // for...in only iterates over defined elements
                            if !matches!(val, Value::Undefined) {
                                indices.push(idx.to_string());
                            }
                        }
                        indices
                    }
                    _ => return Ok(Value::Undefined),
                };

                if declaration_type.is_some() {
                    self.scopes.push(Scope::Block(new_scope_env()));
                    self.set_var(*declaration_type, variable, Value::Undefined);
                }

                let mut last_value = Value::Undefined;
                for key in keys {
                    let key_value = Value::String(key.clone());
                    self.set_var(*declaration_type, variable, key_value);

                    last_value = match self.eval_node(body) {
                        Err(Control::Break(break_label)) if break_label == *label => {
                            if declaration_type.is_some() {
                                self.scopes.pop();
                            }
                            return Ok(self.previous_value.clone());
                        }
                        Err(Control::Continue(continue_label)) if continue_label == *label => {
                            continue;
                        }
                        result => result,
                    }?;
                    self.previous_value = last_value.clone();
                }

                if declaration_type.is_some() {
                    self.scopes.pop();
                }
                Ok(last_value)
            }
            AstNode::ForOf {
                label,
                variable,
                declaration_type,
                iterable,
                body,
            } => {
                let iterable_value = self.eval_node(iterable)?;

                // Get the values to iterate over
                let values = match &iterable_value {
                    Value::Array(arr) => arr.borrow().clone(),
                    _ => return Ok(Value::Undefined),
                };

                if declaration_type.is_some() {
                    self.scopes.push(Scope::Block(new_scope_env()));
                    self.set_var(*declaration_type, variable, Value::Undefined);
                }

                let mut last_value = Value::Undefined;
                for value in values {
                    self.set_var(*declaration_type, variable, value);

                    last_value = match self.eval_node(body) {
                        Err(Control::Break(break_label)) if break_label == *label => {
                            if declaration_type.is_some() {
                                self.scopes.pop();
                            }
                            return Ok(self.previous_value.clone());
                        }
                        Err(Control::Continue(continue_label)) if continue_label == *label => {
                            continue;
                        }
                        result => result,
                    }?;
                    self.previous_value = last_value.clone();
                }

                if declaration_type.is_some() {
                    self.scopes.pop();
                }
                Ok(last_value)
            }
            AstNode::Continue(continue_label) => Err(Control::Continue(continue_label.clone())),
            AstNode::Break(label) => Err(Control::Break(label.clone())),
            AstNode::Return(value) => {
                let ret_value = if let Some(ret_node) = value {
                    self.eval_node(ret_node)?
                } else {
                    Value::Undefined
                };
                Err(Control::Return(ret_value))
            }
            AstNode::FunctionDeclaration { .. } => {
                // Already hoisted before block execution; this is a no-op
                Ok(Value::Undefined)
            }

            AstNode::Comma(nodes) => {
                for node in nodes {
                    self.previous_value = self.eval_node(node)?;
                }
                Ok(self.previous_value.clone())
            }

            AstNode::Value(Value::Function(template, _)) => {
                // Capture lexical scope at function-creation time
                let captured = self.capture_closure();
                Ok(Value::Function(template.clone(), Box::new(captured)))
            }
            AstNode::Value(value) => Ok(value.clone()),
            AstNode::ArrayLiteral(nodes) => {
                let mut elements = Vec::new();
                for node in nodes {
                    elements.push(self.eval_node(node)?);
                }
                Ok(Value::Array(ArrayValue {
                    elements: Rc::new(RefCell::new(elements)),
                }))
            }
            AstNode::ObjectLiteral(properties) => {
                let mut obj = IndexMap::new();
                for (property_key, value_node) in properties {
                    let key_str = match property_key {
                        ObjectProperty::Literal(key) => key.clone(),
                        ObjectProperty::Computed(key_expr) => {
                            let computed_key = self.eval_node(key_expr)?;
                            computed_key.to_string()
                        }
                    };
                    let value = self.eval_node(value_node)?;
                    obj.insert(key_str, value);
                }
                Ok(Value::Object(ObjectValue {
                    properties: Rc::new(RefCell::new(obj)),
                }))
            }
            AstNode::Variable(variable) => match self.get_var(variable) {
                Some(value) => Ok(value.clone()),
                None => Err(Control::Error(format!(
                    "Interpreter: variable {variable} doesn't exists"
                ))),
            },
            AstNode::FunctionCall(function, arguments) => {
                // Method call dispatch for built-in types
                if let AstNode::GetProperty(object_node, property_node) = &**function
                    && let AstNode::Value(Value::String(method_name)) = &**property_node
                {
                    let object_value = self.eval_node(object_node)?;

                    // String built-in methods
                    if let Value::String(s) = &object_value {
                        let s = s.clone();
                        let method = method_name.clone();
                        let mut arg_values = Vec::new();
                        for arg in arguments {
                            arg_values.push(self.eval_node(arg)?);
                        }
                        if let Some(result) = crate::builtins::string::call_method(&s, &method, &arg_values) {
                            return Ok(result);
                        }
                        return Ok(Value::Undefined);
                    }

                    // Array built-in methods
                    if let Value::Array(arr) = object_value.clone() {
                        let method = method_name.clone();
                        let mut arg_values = Vec::new();
                        for arg in arguments {
                            arg_values.push(self.eval_node(arg)?);
                        }
                        if let Some(result) = crate::builtins::array::call_method(arr, &method, arg_values, &mut |func, this, args| self.call_function_value(func, this, args)) {
                            return result;
                        }
                        return Ok(Value::Undefined);
                    }

                    // Number instance methods
                    if let Value::Number(n) = object_value.clone() {
                        let method = method_name.as_str();
                        let mut arg_values = Vec::new();
                        for arg in arguments {
                            arg_values.push(self.eval_node(arg)?);
                        }
                        if let Some(result) = crate::builtins::number::call_method(n, method, &arg_values) {
                            return result.map_err(Control::Error);
                        }
                        return Ok(Value::Undefined);
                    }

                    // Object instance methods
                    if let Value::Object(obj) = &object_value {
                        let mut arg_values = Vec::new();
                        for arg in arguments {
                            arg_values.push(self.eval_node(arg)?);
                        }
                        match method_name.as_str() {
                            "hasOwnProperty" => {
                                let key = arg_values.first().map(|v| v.to_string()).unwrap_or_default();
                                return Ok(Value::Boolean(obj.borrow().contains_key(&key) && !key.starts_with("__")));
                            }
                            "toString" => return Ok(Value::String(String::from("[object Object]"))),
                            "valueOf" => return Ok(Value::Object(obj.clone())),
                            _ => {}
                        }
                    }
                }

                // Check if this is a method call (object.method())
                let (func_value, this_value) =
                    if let AstNode::GetProperty(object_node, _) = &**function {
                        let this_obj = self.eval_node(object_node)?;
                        let func = self.eval_node(function)?;
                        (func, this_obj)
                    } else {
                        let func = self.eval_node(function)?;
                        (func, Value::Undefined)
                    };

                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.eval_node(arg)?);
                }
                self.call_function_value(func_value, this_value, arg_values)
            }

            AstNode::Assign(declaration_type, lhs, rhs) => self.assign(*declaration_type, lhs, rhs),
            AstNode::AddAssign(lhs, rhs) => self.op_assign(lhs, rhs, |a, b| a + b, true),
            AstNode::SubtractAssign(lhs, rhs) => {
                self.op_assign(lhs, rhs, |a, b| a - b, false)
            }
            AstNode::MultiplyAssign(lhs, rhs) => {
                self.op_assign(lhs, rhs, |a, b| a * b, false)
            }
            AstNode::DivideAssign(lhs, rhs) => self.op_assign(lhs, rhs, |a, b| a / b, false),
            AstNode::RemainderAssign(lhs, rhs) => self.op_assign(lhs, rhs, |a, b| a % b, false),
            AstNode::ExponentiationAssign(lhs, rhs) => {
                self.op_assign(lhs, rhs, |a, b| a.powf(b), false)
            }
            AstNode::BitwiseAndAssign(lhs, rhs) => {
                self.binary_op_assign(lhs, rhs, |a, b| a & b)
            }
            AstNode::BitwiseOrAssign(lhs, rhs) => {
                self.binary_op_assign(lhs, rhs, |a, b| a | b)
            }
            AstNode::BitwiseXorAssign(lhs, rhs) => {
                self.binary_op_assign(lhs, rhs, |a, b| a ^ b)
            }
            AstNode::LeftShiftAssign(lhs, rhs) => {
                self.binary_op_assign(lhs, rhs, |a, b| a << b)
            }
            AstNode::SignedRightShiftAssign(lhs, rhs) => {
                self.binary_op_assign(lhs, rhs, |a, b| a >> b)
            }
            AstNode::UnsignedRightShiftAssign(lhs, rhs) => self.binary_op_assign(
                lhs,
                rhs,
                |a, b| ((a as u32) >> (b as u32)) as i32,
            ),
            AstNode::LogicalOrAssign(lhs, rhs) => {
                let lhs_val = self.eval_node(lhs)?;
                if lhs_val.is_truthy() {
                    return Ok(lhs_val);
                }
                let rhs_val = self.eval_node(rhs)?;
                match &**lhs {
                    AstNode::Variable(variable) => self.set_var(None, variable, rhs_val.clone()),
                    _ => {
                        return Err(Control::Error(String::from(
                            "Interpreter: assign lhs is not a variable",
                        )));
                    }
                }
                Ok(rhs_val)
            }
            AstNode::LogicalAndAssign(lhs, rhs) => {
                let lhs_val = self.eval_node(lhs)?;
                if !lhs_val.is_truthy() {
                    return Ok(lhs_val);
                }
                let rhs_val = self.eval_node(rhs)?;
                match &**lhs {
                    AstNode::Variable(variable) => self.set_var(None, variable, rhs_val.clone()),
                    _ => {
                        return Err(Control::Error(String::from(
                            "Interpreter: assign lhs is not a variable",
                        )));
                    }
                }
                Ok(rhs_val)
            }

            AstNode::Ternary {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_value = self.eval_node(condition)?;
                if cond_value.is_truthy() {
                    self.eval_node(then_branch)
                } else {
                    self.eval_node(else_branch)
                }
            }

            AstNode::UnaryMinus(unary) => {
                let val = self.eval_node(unary)?;
                Ok(Value::Number(-val.to_number()))
            }
            AstNode::UnaryLogicalNot(unary) => {
                let val = self.eval_node(unary)?;
                Ok(Value::Boolean(!val.is_truthy()))
            }
            AstNode::UnaryPreIncrement(unary) => match &**unary {
                AstNode::Variable(var_name) => {
                    let n = self.eval_node(unary)?.to_number();
                    let new_value = Value::Number(n + 1.0);
                    self.set_var(None, var_name, new_value.clone());
                    Ok(new_value)
                }
                AstNode::GetProperty(object_node, property_node) => {
                    let object_value = self.eval_node(object_node)?;
                    let property_value = self.eval_node(property_node)?;
                    let n = self
                        .get_property(&object_value, &property_value)
                        .to_number();
                    let new_value = Value::Number(n + 1.0);
                    self.set_property(object_value, property_value, new_value.clone())?;
                    Ok(new_value)
                }
                _ => Err(Control::Error(String::from(
                    "Interpreter: pre-increment can only be applied to variables",
                ))),
            },
            AstNode::UnaryPreDecrement(unary) => match &**unary {
                AstNode::Variable(var_name) => {
                    let n = self.eval_node(unary)?.to_number();
                    let new_value = Value::Number(n - 1.0);
                    self.set_var(None, var_name, new_value.clone());
                    Ok(new_value)
                }
                AstNode::GetProperty(object_node, property_node) => {
                    let object_value = self.eval_node(object_node)?;
                    let property_value = self.eval_node(property_node)?;
                    let n = self
                        .get_property(&object_value, &property_value)
                        .to_number();
                    let new_value = Value::Number(n - 1.0);
                    self.set_property(object_value, property_value, new_value.clone())?;
                    Ok(new_value)
                }
                _ => Err(Control::Error(String::from(
                    "Interpreter: pre-decrement can only be applied to variables",
                ))),
            },
            AstNode::UnaryPostIncrement(unary) => match &**unary {
                AstNode::Variable(var_name) => {
                    let n = self.eval_node(unary)?.to_number();
                    self.set_var(None, var_name, Value::Number(n + 1.0));
                    Ok(Value::Number(n))
                }
                AstNode::GetProperty(object_node, property_node) => {
                    let object_value = self.eval_node(object_node)?;
                    let property_value = self.eval_node(property_node)?;
                    let n = self
                        .get_property(&object_value, &property_value)
                        .to_number();
                    self.set_property(object_value, property_value, Value::Number(n + 1.0))?;
                    Ok(Value::Number(n))
                }
                _ => Err(Control::Error(String::from(
                    "Interpreter: post-increment can only be applied to variables",
                ))),
            },
            AstNode::UnaryPostDecrement(unary) => match &**unary {
                AstNode::Variable(var_name) => {
                    let n = self.eval_node(unary)?.to_number();
                    self.set_var(None, var_name, Value::Number(n - 1.0));
                    Ok(Value::Number(n))
                }
                AstNode::GetProperty(object_node, property_node) => {
                    let object_value = self.eval_node(object_node)?;
                    let property_value = self.eval_node(property_node)?;
                    let n = self
                        .get_property(&object_value, &property_value)
                        .to_number();
                    self.set_property(object_value, property_value, Value::Number(n - 1.0))?;
                    Ok(Value::Number(n))
                }
                _ => Err(Control::Error(String::from(
                    "Interpreter: post-decrement can only be applied to variables",
                ))),
            },
            AstNode::UnaryTypeof(unary) => {
                if let AstNode::Variable(variable) = &**unary
                    && self.get_var(variable).is_none()
                {
                    return Ok(Value::String(String::from("undefined")));
                }
                let val = self.eval_node(unary)?;
                Ok(Value::String(val.typeof_string().to_string()))
            }
            AstNode::UnaryDelete(unary) => match &**unary {
                AstNode::Variable(variable) => {
                    if self.get_var(variable).is_some() {
                        Ok(Value::Boolean(false))
                    } else {
                        Ok(Value::Boolean(true))
                    }
                }
                AstNode::GetProperty(object_node, property_node) => {
                    let object_value = self.eval_node(object_node)?;
                    let property_value = self.eval_node(property_node)?;
                    self.delete_property(object_value, property_value)
                }
                _ => {
                    self.eval_node(unary)?;
                    Ok(Value::Boolean(true))
                }
            },
            AstNode::UnaryVoid(unary) => {
                self.eval_node(unary)?;
                Ok(Value::Undefined)
            }
            AstNode::Throw(expr) => Err(Control::Throw(self.eval_node(expr)?)),

            AstNode::Add(lhs, rhs) => self.arithmetic_op(lhs, rhs, |a, b| a + b, true),
            AstNode::Subtract(lhs, rhs) => {
                self.arithmetic_op(lhs, rhs, |a, b| a - b, false)
            }
            AstNode::Multiply(lhs, rhs) => {
                self.arithmetic_op(lhs, rhs, |a, b| a * b, false)
            }
            AstNode::Divide(lhs, rhs) => self.arithmetic_op(lhs, rhs, |a, b| a / b, false),
            AstNode::Remainder(lhs, rhs) => self.arithmetic_op(lhs, rhs, |a, b| a % b, false),
            AstNode::Exponentiation(lhs, rhs) => {
                self.arithmetic_op(lhs, rhs, |a, b| a.powf(b), false)
            }
            AstNode::BitwiseAnd(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a & b),
            AstNode::BitwiseOr(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a | b),
            AstNode::BitwiseXor(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a ^ b),
            AstNode::LeftShift(lhs, rhs) => self.binary_op(lhs, rhs, |a, b| a << b),
            AstNode::SignedRightShift(lhs, rhs) => {
                self.binary_op(lhs, rhs, |a, b| a >> b)
            }
            AstNode::UnsignedRightShift(lhs, rhs) => self.binary_op(
                lhs,
                rhs,
                |a, b| ((a as u32) >> (b as u32)) as i32,
            ),
            AstNode::BitwiseNot(unary) => {
                let n = self.eval_node(unary)?.to_number() as i32;
                Ok(Value::Number(!n as f64))
            }

            AstNode::Equals(lhs, rhs) => {
                let (lhs_val, rhs_val) = (self.eval_node(lhs)?, self.eval_node(rhs)?);
                Ok(Value::Boolean(lhs_val.loose_equals(&rhs_val)))
            }
            AstNode::StrictEquals(lhs, rhs) => {
                let (lhs_val, rhs_val) = (self.eval_node(lhs)?, self.eval_node(rhs)?);
                Ok(Value::Boolean(lhs_val.js_strict_equals(&rhs_val)))
            }
            AstNode::NotEquals(lhs, rhs) => {
                let (lhs_val, rhs_val) = (self.eval_node(lhs)?, self.eval_node(rhs)?);
                Ok(Value::Boolean(!lhs_val.loose_equals(&rhs_val)))
            }
            AstNode::StrictNotEquals(lhs, rhs) => {
                let (lhs_val, rhs_val) = (self.eval_node(lhs)?, self.eval_node(rhs)?);
                Ok(Value::Boolean(!lhs_val.js_strict_equals(&rhs_val)))
            }
            AstNode::LessThan(lhs, rhs) => self.compare_op(lhs, rhs, |a, b| a < b, |a, b| a < b),
            AstNode::LessThanEquals(lhs, rhs) => {
                self.compare_op(lhs, rhs, |a, b| a <= b, |a, b| a <= b)
            }
            AstNode::GreaterThan(lhs, rhs) => {
                self.compare_op(lhs, rhs, |a, b| a > b, |a, b| a > b)
            }
            AstNode::GreaterThanEquals(lhs, rhs) => {
                self.compare_op(lhs, rhs, |a, b| a >= b, |a, b| a >= b)
            }
            AstNode::In(lhs, rhs) => {
                let property = self.eval_node(lhs)?.to_string();
                let object = self.eval_node(rhs)?;
                Ok(Value::Boolean(self.has_property(&object, &property)))
            }
            AstNode::Instanceof(_lhs, _rhs) => Ok(Value::Boolean(false)),

            AstNode::LogicalAnd(lhs, rhs) => {
                let lhs_val = self.eval_node(lhs)?;
                if !lhs_val.is_truthy() {
                    return Ok(lhs_val);
                }
                self.eval_node(rhs)
            }
            AstNode::LogicalOr(lhs, rhs) => {
                let lhs_val = self.eval_node(lhs)?;
                if lhs_val.is_truthy() {
                    return Ok(lhs_val);
                }
                self.eval_node(rhs)
            }

            AstNode::GetProperty(object_node, property_node) => {
                let object_value = self.eval_node(object_node)?;
                let property_value = self.eval_node(property_node)?;
                match (object_value, property_value) {
                    (Value::String(s), Value::String(property)) => {
                        if property == "length" {
                            Ok(Value::Number(s.chars().count() as f64))
                        } else if let Ok(index) = property.parse::<usize>() {
                            Ok(s.chars()
                                .nth(index)
                                .map(|c| Value::String(c.to_string()))
                                .unwrap_or(Value::Undefined))
                        } else {
                            // String prototype method lookup - return undefined for unknown
                            Ok(Value::Undefined)
                        }
                    }
                    (Value::String(s), Value::Number(index)) => {
                        let idx = index as usize;
                        Ok(s.chars()
                            .nth(idx)
                            .map(|c| Value::String(c.to_string()))
                            .unwrap_or(Value::Undefined))
                    }
                    (Value::Array(elements), Value::Number(index)) => {
                        let idx = index as usize;
                        let arr = elements.borrow();
                        if idx < arr.len() {
                            Ok(arr.get(idx).cloned().unwrap_or(Value::Undefined))
                        } else {
                            Ok(Value::Undefined)
                        }
                    }
                    (Value::Array(elements), Value::String(property)) => {
                        if property == "length" {
                            Ok(Value::Number(elements.borrow().len() as f64))
                        } else if let Ok(index) = property.parse::<usize>() {
                            // String is a numeric index
                            let arr = elements.borrow();
                            if index < arr.len() {
                                Ok(arr.get(index).cloned().unwrap_or(Value::Undefined))
                            } else {
                                Ok(Value::Undefined)
                            }
                        } else {
                            Ok(Value::Undefined)
                        }
                    }
                    (Value::Object(obj), Value::String(property)) => Ok(obj
                        .borrow()
                        .get(&property)
                        .cloned()
                        .unwrap_or(Value::Undefined)),
                    (Value::Object(obj), Value::Number(n)) => {
                        let property = crate::value::number_to_js_string(n);
                        Ok(obj
                            .borrow()
                            .get(&property)
                            .cloned()
                            .unwrap_or(Value::Undefined))
                    }
                    _ => Ok(Value::Undefined),
                }
            }
        }
    }

    // MARK: Function call helper
    // MARK: Hoisting
    fn find_function_scope_env(&self) -> ScopeEnv {
        for scope in self.scopes.iter().rev() {
            if let Scope::Function(env) = scope {
                return env.clone();
            }
        }
        self.global_env.clone()
    }

    // Recursively hoist `var` declarations (crosses block boundaries but not function bodies)
    // into the given env (the nearest enclosing function scope or global scope).
    fn hoist_vars_from_node(&self, node: &AstNode, env: &ScopeEnv) {
        match node {
            AstNode::Assign(Some(DeclarationType::Var), lhs, _) => {
                if let AstNode::Variable(name) = lhs.as_ref() {
                    if !env.borrow().contains_key(name) {
                        env.borrow_mut().insert(name.clone(), Value::Undefined);
                    }
                }
            }
            // Recurse through control structures but NOT into function bodies
            AstNode::Block { nodes, .. } => {
                for n in nodes {
                    self.hoist_vars_from_node(n, env);
                }
            }
            AstNode::If { then_branch, else_branch, .. } => {
                self.hoist_vars_from_node(then_branch, env);
                if let Some(else_b) = else_branch {
                    self.hoist_vars_from_node(else_b, env);
                }
            }
            AstNode::While { body, .. } | AstNode::DoWhile { body, .. } => {
                self.hoist_vars_from_node(body, env);
            }
            AstNode::For { init, body, .. } => {
                if let Some(init_node) = init {
                    self.hoist_vars_from_node(init_node, env);
                }
                self.hoist_vars_from_node(body, env);
            }
            AstNode::ForIn { variable, declaration_type: Some(DeclarationType::Var), body, .. }
            | AstNode::ForOf { variable, declaration_type: Some(DeclarationType::Var), body, .. } => {
                if !env.borrow().contains_key(variable) {
                    env.borrow_mut().insert(variable.clone(), Value::Undefined);
                }
                self.hoist_vars_from_node(body, env);
            }
            AstNode::ForIn { body, .. } | AstNode::ForOf { body, .. } => {
                self.hoist_vars_from_node(body, env);
            }
            AstNode::Switch { clauses, .. } => {
                for clause in clauses {
                    let body = match clause {
                        SwitchClause::Case(_, body) | SwitchClause::Default(body) => body,
                    };
                    self.hoist_vars_from_node(body, env);
                }
            }
            AstNode::Try { try_block, catch_block, finally_block, .. } => {
                self.hoist_vars_from_node(try_block, env);
                if let Some(cb) = catch_block {
                    self.hoist_vars_from_node(cb, env);
                }
                if let Some(fb) = finally_block {
                    self.hoist_vars_from_node(fb, env);
                }
            }
            _ => {}
        }
    }

    // Hoist function declarations at this block level only (not into nested blocks).
    // Closures are captured at the current scope stack state.
    fn hoist_funcs_from_block(&mut self, nodes: &[AstNode], env: &ScopeEnv) {
        for node in nodes {
            if let AstNode::FunctionDeclaration { name, template } = node {
                let captured = self.capture_closure();
                let func_value = Value::Function(template.clone(), Box::new(captured));
                env.borrow_mut().insert(name.clone(), func_value);
            }
        }
    }

    fn call_function_value(
        &mut self,
        func_value: Value,
        this_value: Value,
        arg_values: Vec<Value>,
    ) -> Result<Value, Control> {
        match func_value {
            Value::Function(template, closure_env) => {
                let (arg_names, body) = &*template;
                let func_env = new_scope_env();
                {
                    let mut env = func_env.borrow_mut();
                    env.insert("this".to_string(), this_value);
                    for (i, name) in arg_names.iter().enumerate() {
                        let val = arg_values.get(i).cloned().unwrap_or(Value::Undefined);
                        env.insert(name.clone(), val);
                    }
                    env.insert(
                        "arguments".to_string(),
                        Value::Array(ArrayValue {
                            elements: Rc::new(RefCell::new(arg_values)),
                        }),
                    );
                }
                // Save current scopes, restore lexical closure env, push function scope
                let saved_scopes = std::mem::take(&mut self.scopes);
                self.scopes = closure_env
                    .iter()
                    .map(|(is_fn, env)| {
                        if *is_fn {
                            Scope::Function(env.clone())
                        } else {
                            Scope::Block(env.clone())
                        }
                    })
                    .collect();
                self.scopes.push(Scope::Function(func_env));
                let result = match self.eval_node(body) {
                    Ok(_) => Ok(Value::Undefined),
                    Err(Control::Return(val)) => Ok(val),
                    Err(control) => Err(control),
                };
                self.scopes = saved_scopes;
                result
            }
            Value::NativeFunction(func) => Ok(func(&arg_values)),
            // Callable object: an ObjectValue with a "__call__" NativeFunction property
            Value::Object(obj) => {
                if let Some(Value::NativeFunction(func)) = obj.borrow().get("__call__").cloned() {
                    Ok(func(&arg_values))
                } else {
                    Err(Control::Error(String::from(
                        "Interpreter: trying to call a non-function value",
                    )))
                }
            }
            _ => Err(Control::Error(String::from(
                "Interpreter: trying to call a non-function value",
            ))),
        }
    }

    // MARK: Var get set
    fn capture_closure(&self) -> ClosureEnv {
        self.scopes
            .iter()
            .map(|s| (s.is_function(), s.env_rc().clone()))
            .collect()
    }

    fn get_var(&self, variable: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            let env = scope.env_rc().borrow();
            if let Some(val) = env.get(variable) {
                return Some(val.clone());
            }
        }
        self.global_env.borrow().get(variable).cloned()
    }

    fn set_var(&mut self, declaration_type: Option<DeclarationType>, variable: &str, value: Value) {
        for scope in self.scopes.iter().rev() {
            let is_fn = scope.is_function();
            let env_rc = scope.env_rc().clone();
            let contains = env_rc.borrow().contains_key(variable);
            match scope {
                Scope::Block(_)
                    if matches!(
                        declaration_type,
                        Some(DeclarationType::Let | DeclarationType::Const)
                    ) || contains =>
                {
                    env_rc.borrow_mut().insert(variable.to_string(), value);
                    return;
                }
                Scope::Function(_) if declaration_type.is_some() || contains => {
                    env_rc.borrow_mut().insert(variable.to_string(), value);
                    return;
                }
                _ => {
                    // For var declarations, skip block scopes until we find a function scope
                    if matches!(declaration_type, Some(DeclarationType::Var)) && is_fn {
                        env_rc.borrow_mut().insert(variable.to_string(), value);
                        return;
                    }
                }
            }
        }
        self.global_env
            .borrow_mut()
            .insert(variable.to_string(), value);
    }

    // MARK: Utils
    fn get_property(&self, object: &Value, property: &Value) -> Value {
        match (object, property) {
            (Value::Array(arr), Value::Number(idx)) => arr
                .borrow()
                .get(*idx as usize)
                .cloned()
                .unwrap_or(Value::Undefined),
            (Value::Array(arr), Value::String(prop)) => {
                if prop == "length" {
                    Value::Number(arr.borrow().len() as f64)
                } else if let Ok(idx) = prop.parse::<usize>() {
                    arr.borrow().get(idx).cloned().unwrap_or(Value::Undefined)
                } else {
                    Value::Undefined
                }
            }
            (Value::Object(obj), Value::String(prop)) => {
                obj.borrow().get(prop).cloned().unwrap_or(Value::Undefined)
            }
            _ => Value::Undefined,
        }
    }

    fn set_property(
        &mut self,
        object: Value,
        property: Value,
        value: Value,
    ) -> Result<(), Control> {
        match (object, property) {
            (Value::Array(arr), Value::Number(idx)) => {
                let i = idx as usize;
                let mut b = arr.borrow_mut();
                if i >= b.len() {
                    b.resize(i + 1, Value::Undefined);
                }
                b[i] = value;
                Ok(())
            }
            (Value::Object(obj), Value::String(prop)) => {
                // Silently ignore writes to frozen objects (non-strict mode behavior)
                if matches!(obj.borrow().get("__frozen__"), Some(Value::Boolean(true))) {
                    return Ok(());
                }
                obj.borrow_mut().insert(prop, value);
                Ok(())
            }
            _ => Err(Control::Error(String::from(
                "Interpreter: cannot set property on this value",
            ))),
        }
    }

    fn assign(
        &mut self,
        declaration_type: Option<DeclarationType>,
        lhs: &AstNode,
        rhs: &AstNode,
    ) -> Result<Value, Control> {
        let result = self.eval_node(rhs)?;
        match lhs {
            AstNode::Variable(variable) => {
                self.set_var(declaration_type, variable, result.clone());
                Ok(result)
            }
            AstNode::GetProperty(object_node, property_node) => {
                let object_value = self.eval_node(object_node)?;
                let property_value = self.eval_node(property_node)?;
                match (&object_value, &property_value) {
                    (Value::Array(arr), Value::Number(index)) => {
                        let idx = *index as usize;
                        let mut borrowed_arr = arr.borrow_mut();
                        // Extend array if needed
                        if idx >= borrowed_arr.len() {
                            borrowed_arr.resize(idx + 1, Value::Undefined);
                        }
                        borrowed_arr[idx] = result.clone();
                    }
                    (Value::Array(arr), Value::String(property)) => {
                        if property == "length" {
                            let new_len = result.to_number() as usize;
                            let mut borrowed_arr = arr.borrow_mut();
                            borrowed_arr.truncate(new_len);
                        } else {
                            return Err(Control::Error(String::from(
                                "Interpreter: cannot assign to array property",
                            )));
                        }
                    }
                    (Value::Object(obj), Value::String(property)) => {
                        if !matches!(obj.borrow().get("__frozen__"), Some(Value::Boolean(true))) {
                            obj.borrow_mut().insert(property.clone(), result.clone());
                        }
                    }
                    _ => {
                        return Err(Control::Error(String::from(
                            "Interpreter: invalid property assignment",
                        )));
                    }
                }
                Ok(result)
            }
            _ => Err(Control::Error(String::from(
                "Interpreter: assign lhs is not a variable",
            ))),
        }
    }

    fn arithmetic_op<F>(
        &mut self,
        lhs: &AstNode,
        rhs: &AstNode,
        op: F,
        is_add: bool,
    ) -> Result<Value, Control>
    where
        F: Fn(f64, f64) -> f64,
    {
        let lhs_val = self.eval_node(lhs)?;
        let rhs_val = self.eval_node(rhs)?;

        // ES5: + operator - if either operand is string, concatenate as strings
        if is_add {
            let is_string = matches!(
                (&lhs_val, &rhs_val),
                (Value::String(_), _) | (_, Value::String(_))
            );
            if is_string {
                return Ok(Value::String(format!("{lhs_val}{rhs_val}")));
            }
        }

        // All arithmetic ops coerce both operands to numbers
        let a = lhs_val.to_number();
        let b = rhs_val.to_number();
        Ok(Value::Number(op(a, b)))
    }

    fn binary_op<F>(
        &mut self,
        lhs: &AstNode,
        rhs: &AstNode,
        op: F,
    ) -> Result<Value, Control>
    where
        F: Fn(i32, i32) -> i32,
    {
        let a = self.eval_node(lhs)?.to_number() as i32;
        let b = self.eval_node(rhs)?.to_number() as i32;
        Ok(Value::Number(op(a, b) as f64))
    }

    fn compare_op<F, G>(
        &mut self,
        lhs: &AstNode,
        rhs: &AstNode,
        op: F,
        str_op: G,
    ) -> Result<Value, Control>
    where
        F: Fn(f64, f64) -> bool,
        G: Fn(&str, &str) -> bool,
    {
        let lhs_val = self.eval_node(lhs)?;
        let rhs_val = self.eval_node(rhs)?;
        // ES5: if both are strings, compare lexicographically
        if let (Value::String(a), Value::String(b)) = (&lhs_val, &rhs_val) {
            return Ok(Value::Boolean(str_op(a, b)));
        }
        Ok(Value::Boolean(op(lhs_val.to_number(), rhs_val.to_number())))
    }

    fn has_property(&self, object: &Value, property: &str) -> bool {
        match object {
            Value::Array(elements) => {
                if property == "length" {
                    true
                } else if let Ok(index) = property.parse::<usize>() {
                    index < elements.borrow().len()
                } else {
                    false
                }
            }
            Value::Object(object) => object.borrow().contains_key(property),
            _ => false,
        }
    }

    fn delete_property(
        &mut self,
        object_value: Value,
        property_value: Value,
    ) -> Result<Value, Control> {
        match (object_value, property_value) {
            (Value::Array(array), Value::Number(index)) => {
                let idx = index as usize;
                let mut borrowed = array.borrow_mut();
                if idx < borrowed.len() {
                    borrowed[idx] = Value::Undefined;
                }
                Ok(Value::Boolean(true))
            }
            (Value::Array(array), Value::String(property)) => {
                if property == "length" {
                    return Ok(Value::Boolean(false));
                }
                if let Ok(index) = property.parse::<usize>() {
                    let mut borrowed = array.borrow_mut();
                    if index < borrowed.len() {
                        borrowed[index] = Value::Undefined;
                    }
                }
                Ok(Value::Boolean(true))
            }
            (Value::Object(object), Value::String(property)) => {
                object.borrow_mut().shift_remove(&property);
                Ok(Value::Boolean(true))
            }
            _ => Ok(Value::Boolean(true)),
        }
    }

    fn op_assign<F>(
        &mut self,
        lhs: &AstNode,
        rhs: &AstNode,
        op: F,
        is_add: bool,
    ) -> Result<Value, Control>
    where
        F: Fn(f64, f64) -> f64,
    {
        let lhs_val = self.eval_node(lhs)?;
        let rhs_val = self.eval_node(rhs)?;

        // Handle string concatenation for +=
        let result = if is_add {
            let is_string = matches!(
                (&lhs_val, &rhs_val),
                (Value::String(_), _) | (_, Value::String(_))
            );
            if is_string {
                Value::String(format!("{lhs_val}{rhs_val}"))
            } else {
                Value::Number(op(lhs_val.to_number(), rhs_val.to_number()))
            }
        } else {
            Value::Number(op(lhs_val.to_number(), rhs_val.to_number()))
        };

        match lhs {
            AstNode::Variable(variable) => {
                self.set_var(None, variable, result.clone());
            }
            AstNode::GetProperty(object_node, property_node) => {
                let object_value = self.eval_node(object_node)?;
                let property_value = self.eval_node(property_node)?;
                match (&object_value, &property_value) {
                    (Value::Object(obj), Value::String(prop)) => {
                        obj.borrow_mut().insert(prop.clone(), result.clone());
                    }
                    (Value::Array(arr), Value::Number(index)) => {
                        let idx = *index as usize;
                        let mut borrowed = arr.borrow_mut();
                        if idx >= borrowed.len() {
                            borrowed.resize(idx + 1, Value::Undefined);
                        }
                        borrowed[idx] = result.clone();
                    }
                    _ => {
                        return Err(Control::Error(String::from(
                            "Interpreter: op-assign on invalid property",
                        )));
                    }
                }
            }
            _ => {
                return Err(Control::Error(String::from(
                    "Interpreter: assign lhs is not a variable",
                )));
            }
        }
        Ok(result)
    }

    fn binary_op_assign<F>(
        &mut self,
        lhs: &AstNode,
        rhs: &AstNode,
        op: F,
    ) -> Result<Value, Control>
    where
        F: Fn(i32, i32) -> i32,
    {
        let a = self.eval_node(lhs)?.to_number() as i32;
        let b = self.eval_node(rhs)?.to_number() as i32;
        let result = Value::Number(op(a, b) as f64);
        match lhs {
            AstNode::Variable(variable) => self.set_var(None, variable, result.clone()),
            _ => {
                return Err(Control::Error(String::from(
                    "Interpreter: assign lhs is not a variable",
                )));
            }
        }
        Ok(result)
    }

}
