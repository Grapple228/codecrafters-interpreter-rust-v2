use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::interpreter::Environment;
use crate::{interpreter, Interpreter, Stmt, Token};

use super::Value;
use super::{Error, Result};

pub type CallableFn = fn(interpreter: &Arc<Mutex<Interpreter>>, args: &[Value]) -> Result<Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Callable {
    BuiltIn {
        name: Box<Token>,
        arity: usize,
        function: CallableFn,
    },
    Function {
        declaration: Box<Stmt>,
    },
}

impl Callable {
    pub fn arity(&self) -> usize {
        match self {
            Callable::Function { declaration } => match declaration.as_ref() {
                Stmt::Function { params, .. } => params.len(),
                _ => panic!("not a function"),
            },
            Callable::BuiltIn { arity, .. } => *arity,
        }
    }

    pub fn call(
        &self,
        paren: Token,
        interpreter: &Arc<Mutex<Interpreter>>,
        args: &[Value],
    ) -> Result<Value> {
        match self {
            Callable::Function { declaration, .. } => {
                let mut interpreter = interpreter.lock().unwrap();

                let mut env = Environment::new(Some(interpreter.globals.clone()));

                match declaration.as_ref() {
                    Stmt::Function { name, params, body } => {
                        for (i, arg) in args.iter().enumerate() {
                            env.define(params.get(i).unwrap().lexeme.clone(), Some(arg.to_owned()));
                        }

                        interpreter.execute_block(body, Rc::new(RefCell::new(env)));
                    }
                    _ => panic!("not a function"),
                }

                Ok(Value::Nil)
            }
            Callable::BuiltIn { function, .. } => function(interpreter, args),
        }
    }

    pub fn stringify(&self) -> String {
        match self {
            Callable::Function { declaration } => match declaration.as_ref() {
                Stmt::Function { name, params, body } => format!(
                    "<fn {}({})>",
                    name.lexeme,
                    params
                        .iter()
                        .map(|p| p.lexeme.clone())
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                _ => panic!("not a function"),
            },
            Callable::BuiltIn { name, .. } => format!("<native fn {}>", name),
        }
    }
}
