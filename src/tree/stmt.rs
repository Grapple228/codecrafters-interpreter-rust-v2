use std::{
    borrow::BorrowMut,
    clone,
    sync::{Arc, Mutex},
};

use tracing::debug;
use tracing_subscriber::field::debug;

use crate::{interpreter::Environment, visitor::Acceptor, AstPrinter, Interpreter, Token, Visitor};

use super::Expr;
use crate::interpreter::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print(Box<Expr>),
    Expression(Box<Expr>),
    Var {
        name: Token,
        initializer: Option<Box<Expr>>,
    },
    Block(Vec<Stmt>),
}

impl Acceptor<Result<()>, &Arc<Mutex<Interpreter>>> for Stmt {
    fn accept(&self, visitor: &Arc<Mutex<Interpreter>>) -> Result<()> {
        match self {
            Stmt::Expression(expr) => {
                let _ = expr.accept(visitor)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = expr.accept(visitor)?;
                println!("{}", value.stringify());
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let mut value = None;

                if let Some(initializer) = initializer {
                    value = Some(initializer.accept(visitor)?);
                };

                visitor
                    .lock()
                    .map_err(|e| Error::MutexError(e.to_string()))?
                    .define(name.clone(), value);

                Ok(())
            }
            Stmt::Block(stmts) => {
                let mut interpreter = visitor
                    .lock()
                    .map_err(|e| Error::MutexError(e.to_string()))?;

                let env = Environment::new(Some(interpreter.environment()));
                interpreter.execute_block(stmts, Arc::new(Mutex::new(env)))
            }
        }
    }
}

impl Acceptor<String, &AstPrinter> for Stmt {
    fn accept(&self, visitor: &AstPrinter) -> String {
        match self {
            Stmt::Expression(expr) => expr.accept(visitor),
            Stmt::Print(expr) => {
                format!("print {}", expr.accept(visitor))
            }
            Stmt::Var { name, initializer } => {
                let mut result = String::new();

                result.push_str("var ");
                result.push_str(&name.lexeme);

                if let Some(initializer) = initializer {
                    result.push_str(" = ");
                    result.push_str(&initializer.accept(visitor));
                }

                result
            }
            Stmt::Block(stmts) => {
                let mut result = String::new();

                result.push_str("{\n");

                for stmt in stmts {
                    result.push_str(&stmt.accept(visitor));
                    result.push_str("\n");
                }

                result.push_str("}\n");

                result
            }
        }
    }
}
