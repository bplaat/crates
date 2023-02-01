use crate::parser::Node;
use std::collections::HashMap;

pub struct Interpreter<'a> {
    env: &'a mut HashMap<String, i64>,
}

impl<'a> Interpreter<'a> {
    pub fn new(env: &'a mut HashMap<String, i64>) -> Self {
        Interpreter { env }
    }

    pub fn eval(&mut self, node: Box<Node>) -> i64 {
        match *node {
            Node::Nodes(nodes) => {
                let mut result = 0;
                for node in nodes {
                    result = self.eval(node);
                }
                result
            }
            Node::Number(number) => number,
            Node::Variable(variable) => match self.env.get(&variable) {
                Some(value) => *value,
                None => panic!("Variable {} doesn't exists", variable),
            },
            Node::Assign(lhs, rhs) => {
                let result = self.eval(rhs);
                match *lhs {
                    Node::Variable(variable) => {
                        self.env.insert(variable, result);
                    }
                    _ => panic!("Assign lhs is not a variable"),
                }
                result
            }
            Node::Neg(unary) => -self.eval(unary),
            Node::Add(lhs, rhs) => self.eval(lhs) + self.eval(rhs),
            Node::Sub(lhs, rhs) => self.eval(lhs) - self.eval(rhs),
            Node::Mul(lhs, rhs) => self.eval(lhs) * self.eval(rhs),
            Node::Exp(lhs, rhs) => self.eval(lhs).pow(self.eval(rhs) as u32),
            Node::Div(lhs, rhs) => self.eval(lhs) / self.eval(rhs),
            Node::Mod(lhs, rhs) => self.eval(lhs) % self.eval(rhs),
        }
    }
}
