use std::{
    borrow::BorrowMut,
    sync::{Arc, Mutex},
};

use tracing::debug;
use tracing_subscriber::field::debug;

use crate::{visitor::Acceptor, AstPrinter, Interpreter, Token, Visitor};

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
                    .define(name.lexeme.clone(), value);

                Ok(())
            }
        }
    }
}

impl Acceptor<String, &AstPrinter> for Stmt {
    fn accept(&self, visitor: &AstPrinter) -> String {
        match self {
            Stmt::Expression(expr) => expr.accept(visitor),
            Stmt::Print(expr) => expr.accept(visitor),
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
        }
    }
}
