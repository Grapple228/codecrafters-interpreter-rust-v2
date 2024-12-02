use core::arch;
use std::sync::{Arc, Mutex};

use tracing::debug;

use crate::interpreter::{self, Result};
use crate::{value, TokenType, Value};
use crate::{
    visitor::{self, Acceptor},
    AstPrinter, Interpreter, Token, Visitor,
};

use super::Stmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Option<Value>),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable(Token),
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
}

impl Into<Stmt> for Expr {
    fn into(self) -> Stmt {
        Stmt::Expression(Box::new(self))
    }
}

impl Expr {
    fn parenthesize(visitor: &AstPrinter, name: impl Into<String>, exprs: &[&Box<Expr>]) -> String {
        let mut result = String::new();

        result.push('(');
        result.push_str(&name.into());

        for expr in exprs {
            result.push(' ');
            result.push_str(expr.accept(visitor).as_str());
        }

        result.push(')');

        result.to_string()
    }
}

impl Acceptor<Result<Value>, &Arc<Mutex<Interpreter>>> for Expr {
    fn accept(&self, visitor: &Arc<Mutex<Interpreter>>) -> Result<Value> {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = left.accept(&visitor.clone())?;
                let right = right.accept(visitor)?;

                Ok(left.calculate(Some(&right), operator.clone())?)
            }
            Expr::Grouping(expr) => expr.accept(visitor),
            Expr::Literal(value) => {
                if let Some(value) = value.clone() {
                    Ok(value)
                } else {
                    Ok(Value::Nil)
                }
            }
            Expr::Unary { operator, right } => {
                let value = right.accept(visitor)?;

                Ok(value.calculate(None, operator.clone())?)
            }
            Expr::Variable(name) => {
                let interpreter = visitor
                    .lock()
                    .map_err(|e| interpreter::Error::MutexError(e.to_string()))?;

                let value = interpreter.environment.borrow().get(name.clone())?;

                Ok(value)
            }
            Expr::Assign { name, value } => {
                let value = value.accept(visitor)?;

                let interpreter = visitor
                    .lock()
                    .map_err(|e| interpreter::Error::MutexError(e.to_string()))?;

                interpreter
                    .environment
                    .borrow_mut()
                    .assign(name.clone(), Some(value.clone()));

                Ok(value)
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = left.accept(visitor)?;

                if operator.token_type == TokenType::OR {
                    if left.is_truthy() {
                        return Ok(left);
                    }
                } else {
                    if !left.is_truthy() {
                        return Ok(left);
                    }
                }

                right.accept(visitor)
            }
            Expr::Call {
                callee,
                arguments,
                paren,
            } => {
                let callee = callee.accept(visitor)?;

                let arguments = arguments
                    .iter()
                    .map(|arg| arg.accept(visitor))
                    .collect::<Result<Vec<Value>>>()?;

                if !callee.is_callable() {
                    return Err(value::Error::NotCallable {
                        token: paren.clone(),
                    })?;
                }

                let arity = callee.arity();
                if arguments.len() != arity {
                    return Err(value::Error::InvalidCountOfArguments {
                        token: paren.clone(),
                        count: arguments.len(),
                        expected: arity,
                    })?;
                }

                Ok(callee.call(paren.clone(), visitor, &arguments)?)
            }
        }
    }
}

impl Acceptor<String, &AstPrinter> for Expr {
    fn accept(&self, visitor: &AstPrinter) -> String {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => Self::parenthesize(&visitor, operator.lexeme.clone(), &[left, right]),
            Expr::Grouping(expr) => Self::parenthesize(&visitor, "group", &[expr]),
            Expr::Literal(value) => match value {
                None => panic!("Must not be None"),
                Some(Value::String(s)) => s.clone(),
                Some(Value::Number(n)) => format!("{:?}", n),
                Some(Value::Boolean(b)) => b.to_string(),
                Some(Value::Nil) => String::from("nil"),
                Some(Value::Callable(c)) => c.stringify(),
            },
            Expr::Unary { operator, right } => {
                Self::parenthesize(&visitor, operator.lexeme.clone(), &[right])
            }
            Expr::Variable(name) => format!("{}", name.lexeme),
            Expr::Assign { name, value } => {
                format!("{} = {}", name.lexeme, value.accept(visitor))
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => Self::parenthesize(&visitor, operator.lexeme.clone(), &[left, right]),
            Expr::Call {
                callee,
                arguments,
                paren,
            } => {
                let arguments = arguments
                    .iter()
                    .map(|arg| arg.accept(visitor))
                    .collect::<Vec<String>>()
                    .join(", ");

                format!("{}({})", callee.accept(visitor), arguments)
            }
        }
    }
}
