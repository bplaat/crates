use crate::parser::Node;
use std::collections::HashMap;

struct Interpreter<'a> {
    env: &'a mut HashMap<String, i64>,
}

pub fn interpreter(env: &mut HashMap<String, i64>, node: Box<Node>) -> i64 {
    let mut interpreter = Box::new(Interpreter { env });
    return interpreter_part(&mut interpreter, node);
}

fn interpreter_part(interpreter: &mut Interpreter, node: Box<Node>) -> i64 {
    match *node {
        Node::Nodes(nodes) => {
            let mut result = 0;
            for node in nodes {
                result = interpreter_part(interpreter, node);
            }
            return result;
        }
        Node::Number(number) => number,
        Node::Variable(variable) => match interpreter.env.get(&variable) {
            Some(value) => *value,
            None => panic!("Variable {} doesn't exists", variable),
        },
        Node::Assign(lhs, rhs) => {
            let result = interpreter_part(interpreter, rhs);
            match *lhs {
                Node::Variable(variable) => {
                    interpreter.env.insert(variable, result);
                }
                _ => panic!("Assign lhs is not a variable"),
            }
            return result;
        }
        Node::Neg(unary) => -interpreter_part(interpreter, unary),
        Node::Add(lhs, rhs) => {
            interpreter_part(interpreter, lhs) + interpreter_part(interpreter, rhs)
        }
        Node::Sub(lhs, rhs) => {
            interpreter_part(interpreter, lhs) - interpreter_part(interpreter, rhs)
        }
        Node::Mul(lhs, rhs) => {
            interpreter_part(interpreter, lhs) * interpreter_part(interpreter, rhs)
        }
        Node::Exp(lhs, rhs) => {
            interpreter_part(interpreter, lhs).pow(interpreter_part(interpreter, rhs) as u32)
        }
        Node::Div(lhs, rhs) => {
            interpreter_part(interpreter, lhs) / interpreter_part(interpreter, rhs)
        }
        Node::Mod(lhs, rhs) => {
            interpreter_part(interpreter, lhs) % interpreter_part(interpreter, rhs)
        }
    }
}
