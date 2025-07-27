/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;

use crate::parser::Node;

pub(crate) struct Interpreter<'a> {
    env: &'a mut HashMap<String, i64>,
}

impl<'a> Interpreter<'a> {
    pub(crate) fn new(env: &'a mut HashMap<String, i64>) -> Self {
        Interpreter { env }
    }

    pub(crate) fn eval(&mut self, node: &Node) -> Result<i64, String> {
        match node {
            Node::Nodes(nodes) => {
                let mut result = 0;
                for node in nodes {
                    result = self.eval(node)?;
                }
                Ok(result)
            }
            Node::Number(number) => Ok(*number),
            Node::Variable(variable) => match self.env.get(variable) {
                Some(value) => Ok(*value),
                None => Err(format!("Interpreter: variable {variable} doesn't exists")),
            },
            Node::Assign(lhs, rhs) => {
                let result = self.eval(rhs)?;
                match lhs.as_ref() {
                    Node::Variable(variable) => {
                        self.env.insert(variable.to_string(), result);
                    }
                    _ => return Err(String::from("Interpreter: assign lhs is not a variable")),
                }
                Ok(result)
            }
            Node::Neg(unary) => Ok(-self.eval(unary)?),
            Node::Add(lhs, rhs) => Ok(self.eval(lhs)? + self.eval(rhs)?),
            Node::Sub(lhs, rhs) => Ok(self.eval(lhs)? - self.eval(rhs)?),
            Node::Mul(lhs, rhs) => Ok(self.eval(lhs)? * self.eval(rhs)?),
            Node::Exp(lhs, rhs) => Ok(self.eval(lhs)?.pow(self.eval(rhs)? as u32)),
            Node::Div(lhs, rhs) => {
                let rhs = self.eval(rhs)?;
                Ok(if rhs != 0 { self.eval(lhs)? / rhs } else { 0 })
            }
            Node::Mod(lhs, rhs) => Ok(self.eval(lhs)? % self.eval(rhs)?),
        }
    }
}
