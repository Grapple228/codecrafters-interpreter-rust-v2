use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use tracing::debug;

use crate::interpreter::{self, Environment, Error, Result};
use crate::{
    visitor::{self, Acceptor},
    AstPrinter, Interpreter, Token, Visitor,
};
use crate::{Callable, Value};

use super::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print(Box<Expr>),
    Expression(Box<Expr>),
    Var {
        name: Token,
        initializer: Option<Box<Expr>>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
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

                let interpreter = visitor
                    .lock()
                    .map_err(|e| interpreter::Error::MutexError(e.to_string()))?;

                interpreter
                    .environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), value.clone());

                Ok(())
            }
            Stmt::Block(stmts) => {
                let mut interpreter = visitor
                    .lock()
                    .map_err(|e| Error::MutexError(e.to_string()))?;

                let env = Environment::new(Some(interpreter.environment.clone()));
                interpreter.execute_block(stmts, Rc::new(RefCell::new(env)))
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let value = condition.accept(visitor)?;

                if value.is_truthy() {
                    then_branch.accept(visitor)
                } else if let Some(else_branch) = else_branch {
                    else_branch.accept(visitor)
                } else {
                    Ok(())
                }
            }
            Stmt::While { condition, body } => {
                while condition.accept(visitor)?.is_truthy() {
                    body.accept(visitor)?
                }

                Ok(())
            }
            Stmt::Function { name, params, body } => {
                let mut interpreter = visitor
                    .lock()
                    .map_err(|e| Error::MutexError(e.to_string()))?;

                let value = Value::Callable(Callable::Function {
                    declaration: Box::new(Stmt::Function {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                    }),
                });

                interpreter
                    .environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), Some(value));

                Ok(())
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
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut result = String::new();

                result.push_str("if (");
                result.push_str(&condition.accept(visitor));
                result.push_str(") {");
                result.push_str(&then_branch.accept(visitor));
                result.push_str("}");

                if let Some(else_branch) = else_branch {
                    result.push_str(" else {");
                    result.push_str(&else_branch.accept(visitor));
                    result.push_str("}");
                }

                result
            }
            Stmt::While { condition, body } => {
                let mut result = String::new();

                result.push_str("while (");
                result.push_str(&condition.accept(visitor));
                result.push_str(") {");
                result.push_str(&body.accept(visitor));
                result.push_str("}");

                result
            }
            Stmt::Function { name, params, body } => {
                let mut result = String::new();

                result.push_str("fn ");
                result.push_str(&name.lexeme);

                result.push_str("(");
                result.push_str(
                    &params
                        .iter()
                        .map(|p| p.lexeme.clone())
                        .collect::<Vec<String>>()
                        .join(", "),
                );
                result.push_str(") {");
                for b in body {
                    result.push_str(&b.accept(visitor));
                }
                result.push_str("}");

                result
            }
        }
    }
}
