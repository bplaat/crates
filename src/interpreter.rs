use crate::parser::Node;
use std::collections::HashMap;

pub struct Interpreter<'a> {
    env: &'a mut HashMap<String, i64>,
}

impl<'a> Interpreter<'a> {
    pub fn new(env: &'a mut HashMap<String, i64>) -> Self {
        Interpreter { env }
    }

    pub fn eval(&mut self, node: Box<Node>) -> Result<i64, String> {
        match *node {
            Node::Nodes(nodes) => {
                let mut result = 0;
                for node in nodes {
                    result = self.eval(node)?;
                }
                Ok(result)
            }
            Node::Number(number) => Ok(number),
            Node::Variable(variable) => match self.env.get(&variable) {
                Some(value) => Ok(*value),
                None => Err(format!("Interpreter: variable {} doesn't exists", variable)),
            },
            Node::Assign(lhs, rhs) => {
                let result = self.eval(rhs)?;
                match *lhs {
                    Node::Variable(variable) => {
                        self.env.insert(variable, result);
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
            Node::Div(lhs, rhs) => Ok(self.eval(lhs)? / self.eval(rhs)?),
            Node::Mod(lhs, rhs) => Ok(self.eval(lhs)? % self.eval(rhs)?),
        }
    }
}
