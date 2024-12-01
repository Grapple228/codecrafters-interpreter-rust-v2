use std::{
    borrow::BorrowMut,
    sync::{Arc, Mutex},
};

use crate::{
    value,
    visitor::{Acceptor, Visitor},
    Expr, Stmt, Token, TokenType, Value, W,
};

mod environment;
mod error;

pub use environment::Environment;
pub use error::{Error, Result};

use tracing::{debug, info};
use tracing_subscriber::field::debug;

#[derive(Debug, Default, Clone)]
pub struct Interpreter {
    had_runtime_error: bool,
    // TODO: maybe could be just Environment
    environment: Arc<Mutex<Environment>>,
}

impl Visitor<Result<Value>> for &Arc<Mutex<Interpreter>> {
    fn visit(&self, acceptor: impl Acceptor<Result<Value>, Self>) -> Result<Value> {
        acceptor.accept(&self)
    }
}

impl Visitor<Result<()>> for &Arc<Mutex<Interpreter>> {
    fn visit(&self, acceptor: impl Acceptor<Result<()>, Self>) -> Result<()>
    where
        Self: Sized,
    {
        acceptor.accept(self)
    }
}

// region:    --- Froms

impl From<W<Interpreter>> for Arc<Mutex<Interpreter>> {
    fn from(value: W<Interpreter>) -> Self {
        Arc::new(Mutex::new(value.0))
    }
}

// endregion: --- Froms

impl Interpreter {
    pub fn execute_block(&mut self, stmts: &[Stmt], env: Arc<Mutex<Environment>>) -> Result<()> {
        let prev = self.environment.clone();

        self.environment = env;

        for stmt in stmts {
            match self.execute(stmt.clone()) {
                Ok(_) => {}
                Err(e) => {
                    self.environment = prev;
                    return Err(e);
                }
            }
        }

        self.environment = prev;

        Ok(())
    }

    pub fn environment(&self) -> Arc<Mutex<Environment>> {
        self.environment.clone()
    }

    pub fn define(&mut self, name: Token, value: Option<Value>) -> Result<()> {
        self.environment
            .lock()
            .map_err(|e| Error::MutexError(e.to_string()))?
            .define(name, value);

        Ok(())
    }

    pub fn get(&self, name: Token) -> Result<Value> {
        let value = self
            .environment
            .lock()
            .map_err(|e| Error::MutexError(e.to_string()))?
            .get(name);

        Ok(value?)
    }

    pub fn assign(&mut self, name: Token, value: Option<Value>) -> Result<()> {
        self.environment
            .lock()
            .map_err(|e| Error::MutexError(e.to_string()))?
            .assign(name, value)
            .map_err(Error::from)
    }

    fn execute(&self, stmt: impl Into<Stmt>) -> Result<()> {
        let stmt = stmt.into();

        stmt.accept(&W(self.clone()).into())
    }

    pub fn interpret_expr(&mut self, expr: Expr) -> Result<Value> {
        info!("Interpreting expression...");
        let value = expr.accept(&W(self.clone()).into());

        match value {
            Ok(value) => Ok(value),
            Err(e) => {
                self.had_runtime_error = true;
                Self::error(&e);
                Err(e)
            }
        }
    }

    pub fn interpret_stmt(&mut self, stmts: &[Stmt]) -> Result<()> {
        info!("Interpreting statement...");

        for stmt in stmts {
            let evaluated = stmt.accept(&W(self.clone()).into());

            match evaluated {
                Ok(_) => {}
                Err(e) => {
                    // Stop execution on first error

                    self.had_runtime_error = true;
                    Self::error(&e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    pub fn had_runtime_error(&self) -> bool {
        self.had_runtime_error
    }

    fn error(error: &Error) {
        match error {
            Error::ValueError(error) => match error {
                value::Error::InvalidOperation {
                    left,
                    right,
                    token,
                    message,
                } => crate::report(token.line, message),
                value::Error::InvalidType {
                    left,
                    right,
                    token,
                    message,
                } => crate::report(token.line, message),
                value::Error::ZeroDivision {
                    left,
                    right,
                    token,
                    message,
                } => crate::report(token.line, message),
                value::Error::MustBeNumber {
                    left,
                    token,
                    right,
                    message,
                } => crate::report(token.line, message),
                value::Error::MustBeNumberOrString {
                    left,
                    token,
                    right,
                    message,
                } => crate::report(token.line, message),
            },
            Error::EnvironmentError(error) => match error {
                environment::Error::UndefinedVariable(name) => {
                    crate::report(name.line, format!("Undefined variable '{}'.", name.lexeme))
                }
                environment::Error::MutexError(token, message) => {
                    crate::report(token.line, message)
                }
            },
            Error::MutexError(message) => unreachable!("{}", message),
        }
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Error = Box<dyn std::error::Error>;
    type Result<T> = core::result::Result<T, Error>; // For tests.

    use crate::{interpreter, Token};

    use super::*;

    #[test]
    fn test_evaluate_bool_ok() -> Result<()> {
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Some(Value::Number(3.0)))),
            operator: Token::new(TokenType::BANG_EQUAL, "!=", None, 1),
            right: Box::new(Expr::Literal(Some(Value::Number(3.0)))),
        };

        let mut interpreter = interpreter::Interpreter::default();
        let result = interpreter.interpret_expr(expr)?;

        assert_eq!(result, Value::Boolean(false));

        Ok(())
    }

    #[test]
    fn test_evaluate_number_ok() -> Result<()> {
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Some(Value::Number(3.0)))),
            operator: Token::new(TokenType::PLUS, "+", None, 1),
            right: Box::new(Expr::Literal(Some(Value::Number(3.0)))),
        };

        let mut interpreter = interpreter::Interpreter::default();
        let result = interpreter.interpret_expr(expr)?;

        assert_eq!(result, Value::Number(6.0));

        Ok(())
    }

    #[test]
    fn test_evaluate_string_ok() -> Result<()> {
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Some(Value::String("hello".to_string())))),
            operator: Token::new(TokenType::PLUS, "+", None, 1),
            right: Box::new(Expr::Literal(Some(Value::String("world".to_string())))),
        };

        let mut interpreter = interpreter::Interpreter::default();
        let result = interpreter.interpret_expr(expr)?;

        assert_eq!(result, Value::String("helloworld".to_string()));

        Ok(())
    }

    #[test]
    fn test_evaluate_nil_ok() -> Result<()> {
        let expr = Expr::Literal(None);

        let mut interpreter = interpreter::Interpreter::default();
        let result = interpreter.interpret_expr(expr)?;

        assert_eq!(result, Value::Nil);

        Ok(())
    }

    #[test]
    fn test_evaluate_complex_ok() -> Result<()> {
        // (3 + 4) * (3 + 4) = 49

        let a = Expr::Literal(Some(Value::Number(3.0)));
        let b = Expr::Literal(Some(Value::Number(4.0)));
        let expr = Expr::Binary {
            left: Box::new(a),
            operator: Token::new(TokenType::PLUS, "+", None, 1),
            right: Box::new(b),
        };

        let multiply = Expr::Binary {
            left: Box::new(expr.clone()),
            operator: Token::new(TokenType::STAR, "*", None, 1),
            right: Box::new(expr),
        };

        let mut interpreter = interpreter::Interpreter::default();
        let result = interpreter.interpret_expr(multiply)?;

        assert_eq!(result, Value::Number(49.0));

        Ok(())
    }
}

// endregion: --- Tests
