use crate::{value, visitor::Acceptor, Expr, Stmt, TokenType, Value, Visitor};

mod error;

pub use error::{Error, Result};
use tracing::{debug, info};
use tracing_subscriber::field::debug;

#[derive(Debug, Default, Clone)]
pub struct Interpreter {
    had_runtime_error: bool,
}

impl Visitor<Result<Value>> for &Interpreter {
    fn visit(&self, acceptor: impl Acceptor<Result<Value>, Self>) -> Result<Value> {
        acceptor.accept(&self)
    }
}

impl Visitor<Result<()>> for &Interpreter {
    fn visit(&self, acceptor: impl Acceptor<Result<()>, Self>) -> Result<()> {
        acceptor.accept(&self)
    }
}

impl Interpreter {
    fn execute(&mut self, stmt: impl Into<Stmt>) -> Result<()> {
        let stmt = stmt.into();

        stmt.accept(&self.clone())
    }

    pub fn interpret_expr(&mut self, expr: Expr) -> Result<Value> {
        info!("Interpreting expression...");
        let value = expr.accept(&self.clone());

        match value {
            Ok(value) => Ok(value),
            Err(e) => {
                self.had_runtime_error = true;
                Self::error(e.clone());
                Err(e)
            }
        }
    }

    pub fn interpret_stmt(&mut self, stmts: &[Stmt]) -> Result<()> {
        info!("Interpreting statement...");

        for stmt in stmts {
            let evaluated = stmt.accept(&self.clone());

            match evaluated {
                Ok(_) => {}
                Err(e) => {
                    // Stop execution on first error

                    self.had_runtime_error = true;
                    Self::error(e.clone());
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    pub fn had_runtime_error(&self) -> bool {
        self.had_runtime_error
    }

    fn error(error: Error) {
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
