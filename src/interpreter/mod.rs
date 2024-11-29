use crate::{Expr, TokenType, Value, Visitor};

mod error;

pub use error::{Error, Result};
use tracing::info;

#[derive(Debug, Default)]
pub struct Interpreter {
    had_runtime_error: bool,
}

impl Visitor<Result<Value>> for Interpreter {
    fn visit(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                Ok(left.calculate(Some(&right), operator.token_type.clone())?)
            }
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Literal(value) => {
                if let Some(value) = value.clone() {
                    Ok(value)
                } else {
                    Ok(Value::Nil)
                }
            }
            Expr::Unary { operator, right } => {
                let value = self.evaluate(right)?;

                Ok(value.calculate(None, operator.token_type.clone())?)
            }
        }
    }
}

impl Interpreter {
    fn had_runtime_error(&self) -> bool {
        self.had_runtime_error
    }

    fn evaluate(&self, expr: &Expr) -> Result<Value> {
        expr.accept(self)
    }

    pub fn interpret(&self, expr: Expr) -> Result<Value> {
        info!("Interpreting tokens...");
        let value = self.evaluate(&expr);

        match value {
            Ok(value) => Ok(value),
            Err(e) => {
                Self::error(e.clone());
                Err(e)
            }
        }
    }

    fn error(error: Error) {
        match error {
            Error::ValueError(error) => match error {
                crate::value::Error::InvalidOperation {
                    left,
                    right,
                    operator,
                } => {
                    info!(
                        "Invalid operation: {} {} {}",
                        left,
                        operator,
                        match right {
                            Some(right) => right.to_string(),
                            None => String::from("None"),
                        }
                    );
                }
                crate::value::Error::InvalidType {
                    left,
                    right,
                    operator,
                } => todo!(),
            },
        }
    }
}
