use std::cell::RefCell;
use std::rc::Rc;

use crate::interpreter::{self, Environment, MutEnv};
use crate::{MutInterpreter, Stmt, Token};

use super::Value;
use interpreter::Result;

pub type CallableFn = fn(interpreter: &MutInterpreter, args: &[Value]) -> Result<Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Callable {
    BuiltIn {
        name: Box<Token>,
        arity: usize,
        function: CallableFn,
    },
    Function {
        declaration: Box<Stmt>,
        closure: MutEnv,
    },
}

impl Callable {
    pub fn arity(&self) -> usize {
        match self {
            Callable::Function { declaration, .. } => match declaration.as_ref() {
                Stmt::Function { params, .. } => params.len(),
                _ => panic!("not a function"),
            },
            Callable::BuiltIn { arity, .. } => *arity,
        }
    }

    pub fn call(&self, interpreter: &MutInterpreter, args: &[Value]) -> Result<Value> {
        match self {
            Callable::Function {
                declaration,
                closure,
            } => {
                let mut interpreter = interpreter.borrow_mut();

                let mut env = Environment::new(Some(closure.clone()));

                let result = match declaration.as_ref() {
                    Stmt::Function { params, body, .. } => {
                        for (i, arg) in args.iter().enumerate() {
                            env.define(params.get(i).unwrap().lexeme.clone(), Some(arg.to_owned()));
                        }

                        match interpreter.execute_block(body, Rc::new(RefCell::new(env))) {
                            Ok(_) => Ok(Value::Nil),
                            Err(interpreter::Error::Return(value)) => Ok(value),
                            Err(e) => Err(e),
                        }
                    }
                    _ => panic!("not a function"),
                };

                result
            }
            Callable::BuiltIn { function, .. } => function(interpreter, args),
        }
    }

    pub fn stringify(&self) -> String {
        match self {
            Callable::Function { declaration, .. } => match declaration.as_ref() {
                Stmt::Function { name, .. } => format!("<fn {}>", name.lexeme,),
                _ => panic!("not a function"),
            },
            Callable::BuiltIn { name, .. } => format!("<native fn {}>", name),
        }
    }
}
