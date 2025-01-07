use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    value::{self, CallableFn},
    visitor::{Acceptor, Visitor},
    Callable, Expr, Stmt, Token, TokenType, Value, W,
};

mod builtins;
mod environment;
mod error;

pub use environment::{Environment, MutEnv};
pub use error::{Error, Result};

use tracing::info;

pub type MutInterpreter = Rc<RefCell<Interpreter>>;

#[derive(Debug, Default, Clone)]
pub struct Interpreter {
    had_runtime_error: bool,
    pub environment: MutEnv,
    pub globals: MutEnv,
    pub locals: HashMap<String, usize>,
}

impl Visitor<Result<Value>> for &MutInterpreter {
    fn visit(&self, acceptor: impl Acceptor<Result<Value>, Self>) -> Result<Value>
    where
        Self: Sized,
    {
        acceptor.accept(self)
    }
}

impl Visitor<Result<()>> for &MutInterpreter {
    fn visit(&self, acceptor: impl Acceptor<Result<()>, Self>) -> Result<()>
    where
        Self: Sized,
    {
        acceptor.accept(self)
    }
}

// region:    --- Froms

impl From<W<Interpreter>> for MutInterpreter {
    fn from(value: W<Interpreter>) -> Self {
        Rc::new(RefCell::from(value.0))
    }
}

// endregion: --- Froms

impl Interpreter {
    pub fn default() -> Self {
        let globals = Rc::new(RefCell::new(Environment::default()));

        let mut interpreter = Self {
            globals: globals.clone(),
            environment: globals,
            ..Default::default()
        };

        interpreter.define_natives();

        interpreter
    }

    pub fn look_up_variable(&self, name: &Token) -> Result<Value> {
        let value = if let Some(distance) = self.locals.get(&name.lexeme).cloned() {
            self.environment.borrow().get_at(distance, &name)?
        } else {
            self.globals.borrow().get(&name)?
        };

        Ok(value)
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        if let Some(name) = expr.name() {
            self.locals.insert(name, depth);
        }
    }

    fn define_natives(&mut self) {
        self.define_native("clock", 0, builtins::clock);
        self.define_native("sum", 2, builtins::sum);
    }

    fn define_native(&mut self, name: impl Into<String>, arity: usize, func: CallableFn) {
        let name: String = name.into();

        let value = Value::Callable(Callable::BuiltIn {
            arity,
            name: Box::new(Token::new(TokenType::IDENTIFIER, &name, None, 0)),
            function: func,
        });

        self.globals.borrow_mut().define(&name, Some(value));
    }

    pub fn execute_block(&mut self, stmts: &[Stmt], env: MutEnv) -> Result<()> {
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

    fn execute(&self, stmt: impl Into<Stmt>) -> Result<()> {
        let stmt: Stmt = stmt.into();

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
                value::Error::InvalidOperation { token, message } => {
                    crate::report(token.line, message)
                }
                value::Error::InvalidType { token, message } => crate::report(token.line, message),
                value::Error::ZeroDivision { token, message } => crate::report(token.line, message),
                value::Error::MustBeNumber { token, message } => crate::report(token.line, message),
                value::Error::MustBeNumberOrString { token, message } => {
                    crate::report(token.line, message)
                }
                value::Error::NotCallable { token } => {
                    crate::report(token.line, format!("{} is not callable.", token.lexeme));
                }
                value::Error::InvalidCountOfArguments {
                    token,
                    count,
                    expected,
                } => {
                    crate::report(
                        token.line,
                        format!(
                            "{} expected {} arguments but got {}.",
                            token.lexeme, expected, count
                        ),
                    );
                }
            },
            Error::EnvironmentError(error) => match error {
                environment::Error::UndefinedVariable(name) => {
                    crate::report(name.line, format!("Undefined variable '{}'.", name.lexeme))
                }
                environment::Error::AncestorNotFound(depth, name) => crate::report(
                    name.line,
                    format!(
                        "Ancestor with {} not found at depth {}.",
                        name.lexeme, depth
                    ),
                ),
            },
            Error::MutexError(message) => unreachable!("{}", message),
            Error::Return(_) => unreachable!(),
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
