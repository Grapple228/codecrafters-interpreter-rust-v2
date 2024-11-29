mod error;

pub use error::{Error, Result};

use crate::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            _ => true,
        }
    }

    pub fn is_equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Number(n1), Value::Number(n2)) => n1 == n2,
            (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }

    /// `other` is optional. Needed only for uperations that can be done with one operand
    /// like `!` or `-`
    pub fn calculate(&self, other: Option<&Value>, operator: TokenType) -> Result<Self> {
        match operator {
            TokenType::MINUS => match (self, other) {
                (Value::Number(a), Some(Value::Number(b))) => Ok(Value::Number(a - b)),
                (Value::Number(a), None) => Ok(Value::Number(-a)),
                _ => Err(Error::InvalidOperation {
                    left: self.clone(),
                    right: other.cloned(),
                    operator,
                }),
            },
            TokenType::PLUS => match (self, other) {
                (Value::Number(a), Some(Value::Number(b))) => Ok(Value::Number(a + b)),
                (Value::String(a), Some(Value::String(b))) => {
                    Ok(Value::String(format!("{}{}", a, b)))
                }
                (Value::String(a), None) => Ok(Value::String(a.clone())),
                _ => Err(Error::InvalidOperation {
                    left: self.clone(),
                    right: other.cloned(),
                    operator,
                }),
            },
            TokenType::SLASH => {
                if let (Value::Number(a), Some(Value::Number(b))) = (self, other) {
                    if *b == 0.0 {
                        Err(Error::InvalidOperation {
                            left: self.clone(),
                            right: other.cloned(),
                            operator,
                        })
                    } else {
                        Ok(Value::Number(a / b))
                    }
                } else {
                    Err(Error::InvalidOperation {
                        left: self.clone(),
                        right: other.cloned(),
                        operator,
                    })
                }
            }
            TokenType::STAR => match (self, other) {
                (Value::Number(a), Some(Value::Number(b))) => Ok(Value::Number(a * b)),
                _ => Err(Error::InvalidOperation {
                    left: self.clone(),
                    right: other.cloned(),
                    operator,
                }),
            },
            TokenType::BANG => {
                if other.is_none() {
                    Ok(Value::Boolean(!self.is_truthy()))
                } else {
                    Err(Error::InvalidOperation {
                        left: self.clone(),
                        right: other.cloned(),
                        operator,
                    })
                }
            }
            TokenType::BANG_EQUAL => {
                if let (left, Some(right)) = (self, other) {
                    Ok(Value::Boolean(!left.is_equal(right)))
                } else {
                    Err(Error::InvalidOperation {
                        left: self.clone(),
                        right: other.cloned(),
                        operator,
                    })
                }
            }
            TokenType::EQUAL_EQUAL => {
                if let (left, Some(right)) = (self, other) {
                    Ok(Value::Boolean(left.is_equal(right)))
                } else {
                    Err(Error::InvalidOperation {
                        left: self.clone(),
                        right: other.cloned(),
                        operator,
                    })
                }
            }
            TokenType::GREATER => match (self, other) {
                (Value::Number(a), Some(Value::Number(b))) => Ok(Value::Boolean(a > b)),
                (Value::String(a), Some(Value::String(b))) => Ok(Value::Boolean(a > b)),
                _ => Err(Error::InvalidOperation {
                    left: self.clone(),
                    right: other.cloned(),
                    operator,
                }),
            },
            TokenType::GREATER_EQUAL => match (self, other) {
                (Value::Number(a), Some(Value::Number(b))) => Ok(Value::Boolean(a >= b)),
                (Value::String(a), Some(Value::String(b))) => Ok(Value::Boolean(a >= b)),
                _ => Err(Error::InvalidOperation {
                    left: self.clone(),
                    right: other.cloned(),
                    operator,
                }),
            },
            TokenType::LESS => match (self, other) {
                (Value::Number(a), Some(Value::Number(b))) => Ok(Value::Boolean(a < b)),
                (Value::String(a), Some(Value::String(b))) => Ok(Value::Boolean(a < b)),
                _ => Err(Error::InvalidOperation {
                    left: self.clone(),
                    right: other.cloned(),
                    operator,
                }),
            },
            TokenType::LESS_EQUAL => match (self, other) {
                (Value::Number(a), Some(Value::Number(b))) => Ok(Value::Boolean(a <= b)),
                (Value::String(a), Some(Value::String(b))) => Ok(Value::Boolean(a <= b)),
                _ => Err(Error::InvalidOperation {
                    left: self.clone(),
                    right: other.cloned(),
                    operator,
                }),
            },
            _ => Err(Error::InvalidOperation {
                left: self.clone(),
                right: other.cloned(),
                operator,
            }),
        }
    }
}

impl core::fmt::Display for Value {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        match self {
            Value::String(s) => write!(fmt, "{}", s),
            Value::Number(n) => write!(fmt, "{:?}", n),
            Value::Boolean(b) => write!(fmt, "{}", b),
            Value::Nil => write!(fmt, "nil"),
        }
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Error = Box<dyn std::error::Error>;
    type Result<T> = core::result::Result<T, Error>; // For tests.

    use super::*;

    #[test]
    fn test_value_truthy_ok() -> Result<()> {
        assert!(!Value::Nil.is_truthy());
        assert!(Value::Boolean(true).is_truthy());
        assert!(!Value::Boolean(false).is_truthy());
        assert!(Value::Number(0.0).is_truthy());
        assert!(Value::String(String::new()).is_truthy());

        Ok(())
    }

    #[test]
    fn test_value_equal_check_ok() -> Result<()> {
        todo!();

        Ok(())
    }
}

// endregion: --- Tests
