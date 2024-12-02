mod callable;
mod error;

pub use callable::{Callable, CallableFn};
pub use error::{Error, Result};

use crate::{extensions::StringExt, interpreter, MutInterpreter, Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
    Callable(Callable),
}

impl Value {
    pub fn arity(&self) -> usize {
        match self {
            Value::Callable(callable) => callable.arity(),
            _ => 0,
        }
    }

    pub fn is_callable(&self) -> bool {
        match self {
            Value::Callable(_) => true,
            _ => false,
        }
    }

    pub fn call(
        &self,
        paren: Token,
        interpreter: &MutInterpreter,
        args: &[Value],
    ) -> std::result::Result<Value, interpreter::Error> {
        match self {
            Value::Callable(callable) => callable.call(interpreter, args),
            _ => {
                return Err(Error::NotCallable {
                    token: paren.clone(),
                })?;
            }
        }
    }

    pub fn stringify(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => {
                let mut s = n.to_string();

                if s.ends_with(".0") {
                    s = s.substring(0, s.len() - 2);
                }
                return s;
            }
            Value::Boolean(b) => b.to_string(),
            Value::Nil => "nil".to_string(),
            Value::Callable(callable) => callable.stringify(),
        }
    }

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
    pub fn calculate(&self, other: Option<&Value>, token: Token) -> Result<Self> {
        let operator = token.clone().token_type;
        // TODO: Check error messages

        match operator {
            // -- Basic calculations
            TokenType::MINUS => match (self, other) {
                (Value::Number(a), Some(Value::Number(b))) => Ok(Value::Number(a - b)),
                (Value::Number(a), None) => Ok(Value::Number(-a)),
                (_, None) => Err(Error::MustBeNumber {
                    token: token.clone(),
                    message: String::from("Operand must be a number."),
                }),
                _ => Err(Error::InvalidType {
                    token,
                    message: String::from("Operation must be done with numbers."),
                }),
            },
            TokenType::PLUS => match (self, other) {
                (Value::Number(a), Some(Value::Number(b))) => Ok(Value::Number(a + b)),
                (Value::String(a), Some(Value::String(b))) => {
                    Ok(Value::String(format!("{}{}", a, b)))
                }
                // (Value::String(a), None) => Ok(Value::String(a.clone())),
                _ => Err(Error::InvalidType {
                    token,
                    message: String::from("Operation must be done with numbers or strings."),
                }),
            },
            TokenType::SLASH => {
                if let (Value::Number(a), Some(Value::Number(b))) = (self, other) {
                    if *b == 0.0 {
                        Err(Error::ZeroDivision {
                            token,
                            message: String::from("Cannot divide by zero."),
                        })
                    } else {
                        Ok(Value::Number(a / b))
                    }
                } else {
                    Err(Error::InvalidType {
                        token,
                        message: String::from("Operation must be done with numbers."),
                    })
                }
            }
            TokenType::STAR => match (self, other) {
                (Value::Number(a), Some(Value::Number(b))) => Ok(Value::Number(a * b)),
                _ => Err(Error::InvalidType {
                    token,
                    message: String::from("Operation must be done with numbers."),
                }),
            },

            // - Bang
            TokenType::BANG => {
                if other.is_none() {
                    Ok(Value::Boolean(!self.is_truthy()))
                } else {
                    Err(Error::InvalidOperation {
                        token,
                        message: String::from("Operation must be done with one operand."),
                    })
                }
            }

            // - Comparisons
            TokenType::EQUAL_EQUAL => match (self, other) {
                (left, Some(right)) => Ok(Value::Boolean(left.is_equal(right))),
                _ => Err(Error::InvalidOperation {
                    token,
                    message: String::from("Operation must be done with two operands."),
                }),
            },
            TokenType::BANG_EQUAL => match (self, other) {
                (left, Some(right)) => Ok(Value::Boolean(!left.is_equal(right))),
                _ => Err(Error::InvalidOperation {
                    token,
                    message: String::from("Operation must be done with two operands."),
                }),
            },
            TokenType::GREATER => match (self, other) {
                (Value::Number(a), Some(Value::Number(b))) => Ok(Value::Boolean(a > b)),
                (Value::String(a), Some(Value::String(b))) => Ok(Value::Boolean(a > b)),
                _ => Err(Error::InvalidOperation {
                    token,
                    message: String::from("Operation must be done with two operands."),
                }),
            },
            TokenType::GREATER_EQUAL => match (self, other) {
                (Value::Number(a), Some(Value::Number(b))) => Ok(Value::Boolean(a >= b)),
                (Value::String(a), Some(Value::String(b))) => Ok(Value::Boolean(a >= b)),
                _ => Err(Error::InvalidOperation {
                    token,
                    message: String::from("Operation must be done with two operands."),
                }),
            },
            TokenType::LESS => match (self, other) {
                (Value::Number(a), Some(Value::Number(b))) => Ok(Value::Boolean(a < b)),
                (Value::String(a), Some(Value::String(b))) => Ok(Value::Boolean(a < b)),
                _ => Err(Error::InvalidOperation {
                    token,
                    message: String::from("Operation must be done with two operands."),
                }),
            },
            TokenType::LESS_EQUAL => match (self, other) {
                (Value::Number(a), Some(Value::Number(b))) => Ok(Value::Boolean(a <= b)),
                (Value::String(a), Some(Value::String(b))) => Ok(Value::Boolean(a <= b)),
                _ => Err(Error::InvalidOperation {
                    token,
                    message: String::from("Operation must be done with two operands."),
                }),
            },

            _ => Err(Error::InvalidOperation {
                token,
                message: String::from("Invalid operation."),
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
            Value::Callable(c) => write!(fmt, "{}", c.stringify()),
        }
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Error = Box<dyn std::error::Error>;
    type Result<T> = core::result::Result<T, Error>; // For tests.

    use super::*;

    fn create_token(token_type: TokenType) -> Token {
        Token::new(token_type.clone(), token_type.to_string(), None, 1)
    }

    #[test]
    /// Tests what prints to console by display
    fn test_value_display_ok() -> Result<()> {
        let str = Value::String("hello".to_string());
        let num = Value::Number(6.0);
        let num_with_dec = Value::Number(6.02);
        let bool_true = Value::Boolean(true);
        let bool_false = Value::Boolean(false);
        let nil = Value::Nil;

        assert_eq!("hello", format!("{}", str));
        assert_eq!("6.0", format!("{}", num));
        assert_eq!("6.02", format!("{}", num_with_dec));
        assert_eq!("true", format!("{}", bool_true));
        assert_eq!("false", format!("{}", bool_false));
        assert_eq!("nil", format!("{}", nil));

        Ok(())
    }

    #[test]
    /// Tests what returns from stringify for user display
    fn test_value_stringify_ok() -> Result<()> {
        let str = Value::String("hello".to_string());
        let num = Value::Number(6.0);
        let num_with_dec = Value::Number(6.02);
        let bool_true = Value::Boolean(true);
        let bool_false = Value::Boolean(false);
        let nil = Value::Nil;

        assert_eq!("hello", str.stringify());
        assert_eq!("6", num.stringify());
        assert_eq!("6.02", num_with_dec.stringify());
        assert_eq!("true", bool_true.stringify());
        assert_eq!("false", bool_false.stringify());
        assert_eq!("nil", nil.stringify());

        Ok(())
    }

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
    fn test_value_operation_negation_ok() -> Result<()> {
        let negate = |left: Value, right: Option<&Value>| {
            let token = create_token(TokenType::MINUS);
            left.calculate(right, token)
        };

        let b_true = Value::Boolean(true);
        let b_false = Value::Boolean(false);
        let a_nubmer = Value::Number(6.0);
        let a_string = Value::String("hello".to_string());
        let nil = Value::Nil;

        // Correctly negates value
        assert!(negate(b_true.clone(), None).is_err());
        assert!(negate(b_false.clone(), None).is_err());
        assert_eq!(negate(a_nubmer.clone(), None)?, Value::Number(-6.0));
        assert!(negate(a_string.clone(), None).is_err());
        assert!(negate(nil.clone(), None).is_err());

        Ok(())
    }

    #[test]
    fn test_value_operation_basic_calculations_ok() -> Result<()> {
        let b_true = &Value::Boolean(true);
        let a_nubmer = &Value::Number(6.0);
        let a_string = &Value::String("hello".to_string());
        let nil = &Value::Nil;

        // error if bool
        assert!(b_true
            .calculate(Some(b_true), create_token(TokenType::PLUS))
            .is_err());
        assert!(b_true
            .calculate(Some(b_true), create_token(TokenType::MINUS))
            .is_err());
        assert!(b_true
            .calculate(Some(b_true), create_token(TokenType::STAR))
            .is_err());
        assert!(b_true
            .calculate(Some(b_true), create_token(TokenType::SLASH))
            .is_err());

        // error if nil
        assert!(nil
            .calculate(Some(nil), create_token(TokenType::PLUS))
            .is_err());
        assert!(nil
            .calculate(Some(nil), create_token(TokenType::MINUS))
            .is_err());
        assert!(nil
            .calculate(Some(nil), create_token(TokenType::STAR))
            .is_err());
        assert!(nil
            .calculate(Some(nil), create_token(TokenType::SLASH))
            .is_err());

        // region:    --- STRING

        assert_eq!(
            a_string.calculate(Some(a_string), create_token(TokenType::PLUS))?,
            Value::String(format!("{}{}", a_string, a_string))
        );
        assert!(a_string
            .calculate(Some(a_string), create_token(TokenType::MINUS))
            .is_err());
        assert!(a_string
            .calculate(Some(a_string), create_token(TokenType::STAR))
            .is_err());
        assert!(a_string
            .calculate(Some(a_string), create_token(TokenType::SLASH))
            .is_err());
        // endregion: --- STRING

        // region:    --- NUMBER

        assert_eq!(
            a_nubmer.calculate(Some(a_nubmer), create_token(TokenType::PLUS))?,
            Value::Number(12.0)
        );
        assert_eq!(
            a_nubmer.calculate(Some(a_nubmer), create_token(TokenType::MINUS))?,
            Value::Number(0.0)
        );
        assert_eq!(
            a_nubmer.calculate(Some(a_nubmer), create_token(TokenType::STAR))?,
            Value::Number(36.0)
        );
        assert_eq!(
            a_nubmer.calculate(Some(a_nubmer), create_token(TokenType::SLASH))?,
            Value::Number(1.0)
        );
        assert!(a_nubmer
            .calculate(Some(a_string), create_token(TokenType::PLUS))
            .is_err());
        assert!(a_nubmer
            .calculate(Some(&Value::Number(0.0)), create_token(TokenType::SLASH))
            .is_err());
        // endregion: --- NUMBER

        Ok(())
    }

    #[test]
    fn test_value_operation_comparison_ok() -> Result<()> {
        let b_true = Value::Boolean(true);
        let b_false = Value::Boolean(false);
        let a_nubmer = Value::Number(6.0);
        let a_string = Value::String("hello".to_string());
        let nil = Value::Nil;

        // region:    --- EQUAL

        assert_eq!(
            b_true.calculate(Some(&b_true), create_token(TokenType::EQUAL_EQUAL))?,
            Value::Boolean(true)
        );
        assert_eq!(
            b_true.calculate(Some(&b_false), create_token(TokenType::EQUAL_EQUAL))?,
            Value::Boolean(false)
        );
        assert_eq!(
            b_true.calculate(Some(&a_nubmer), create_token(TokenType::EQUAL_EQUAL))?,
            Value::Boolean(false)
        );
        assert_eq!(
            b_true.calculate(Some(&a_string), create_token(TokenType::EQUAL_EQUAL))?,
            Value::Boolean(false)
        );
        assert_eq!(
            b_true.calculate(Some(&nil), create_token(TokenType::EQUAL_EQUAL))?,
            Value::Boolean(false)
        );

        // endregion: --- EQUAL

        // region:    --- BANG EQUAL

        assert_eq!(
            b_true.calculate(Some(&b_true), create_token(TokenType::BANG_EQUAL))?,
            Value::Boolean(false)
        );
        assert_eq!(
            b_true.calculate(Some(&b_false), create_token(TokenType::BANG_EQUAL))?,
            Value::Boolean(true)
        );
        assert_eq!(
            b_true.calculate(Some(&a_nubmer), create_token(TokenType::BANG_EQUAL))?,
            Value::Boolean(true)
        );
        assert_eq!(
            b_true.calculate(Some(&a_string), create_token(TokenType::BANG_EQUAL))?,
            Value::Boolean(true)
        );
        assert_eq!(
            b_true.calculate(Some(&nil), create_token(TokenType::BANG_EQUAL))?,
            Value::Boolean(true)
        );
        // endregion: --- BANG EQUAL

        // region:    --- GREATER

        assert!(b_true
            .calculate(Some(&b_true), create_token(TokenType::GREATER))
            .is_err());
        assert!(nil
            .calculate(Some(&nil), create_token(TokenType::GREATER))
            .is_err());
        assert_eq!(
            a_string.calculate(
                Some(&Value::String("world".to_string())),
                create_token(TokenType::GREATER)
            )?,
            Value::Boolean(false)
        );
        assert_eq!(
            a_nubmer.calculate(Some(&Value::Number(6.0)), create_token(TokenType::GREATER))?,
            Value::Boolean(false)
        );

        // less
        assert!(b_true
            .calculate(Some(&b_true), create_token(TokenType::LESS))
            .is_err());
        assert!(nil
            .calculate(Some(&nil), create_token(TokenType::LESS))
            .is_err());
        assert_eq!(
            a_string.calculate(
                Some(&Value::String("world".to_string())),
                create_token(TokenType::LESS)
            )?,
            Value::Boolean(true)
        );
        assert_eq!(
            a_nubmer.calculate(Some(&Value::Number(6.0)), create_token(TokenType::LESS))?,
            Value::Boolean(false)
        );
        // endregion: --- GREATER

        // region:    --- GREATER EQUAL

        assert!(b_true
            .calculate(Some(&b_true), create_token(TokenType::GREATER_EQUAL))
            .is_err());
        assert!(nil
            .calculate(Some(&nil), create_token(TokenType::GREATER_EQUAL))
            .is_err());
        assert_eq!(
            a_string.calculate(
                Some(&Value::String("world".to_string())),
                create_token(TokenType::GREATER_EQUAL)
            )?,
            Value::Boolean(false)
        );
        assert_eq!(
            a_nubmer.calculate(
                Some(&Value::Number(6.0)),
                create_token(TokenType::GREATER_EQUAL)
            )?,
            Value::Boolean(true)
        );
        // endregion: --- GREATER EQUAL

        // region:    --- LESS EQUAL

        assert!(b_true
            .calculate(Some(&b_true), create_token(TokenType::GREATER_EQUAL))
            .is_err());
        assert!(nil
            .calculate(Some(&nil), create_token(TokenType::GREATER_EQUAL))
            .is_err());
        assert_eq!(
            a_string.calculate(
                Some(&Value::String("world".to_string())),
                create_token(TokenType::GREATER_EQUAL)
            )?,
            Value::Boolean(false)
        );
        assert_eq!(
            a_nubmer.calculate(
                Some(&Value::Number(6.0)),
                create_token(TokenType::GREATER_EQUAL)
            )?,
            Value::Boolean(true)
        );

        // endregion: --- LESS EQUAL

        Ok(())
    }

    #[test]
    fn test_value_operation_bang_ok() -> Result<()> {
        let bang = |left: Value, right: Option<&Value>| {
            let token: Token = Token::new(TokenType::BANG, "!", None, 1);
            left.calculate(right, token)
        };

        let b_true = Value::Boolean(true);
        let b_false = Value::Boolean(false);
        let a_nubmer = Value::Number(6.0);
        let a_string = Value::String("hello".to_string());
        let nil = Value::Nil;

        // Correctly bang value
        assert_eq!(bang(b_true.clone(), None)?, Value::Boolean(false));
        assert_eq!(bang(b_false.clone(), None)?, Value::Boolean(true));
        assert_eq!(bang(a_nubmer.clone(), None)?, Value::Boolean(false));
        assert_eq!(bang(a_string.clone(), None)?, Value::Boolean(false));
        assert_eq!(bang(nil.clone(), None)?, Value::Boolean(true));

        // Must take only one operand
        assert!(bang(b_true.clone(), Some(&b_true)).is_err());
        assert!(bang(b_true.clone(), Some(&b_false)).is_err());
        assert!(bang(b_true.clone(), Some(&a_nubmer)).is_err());
        assert!(bang(b_true.clone(), Some(&a_string)).is_err());
        assert!(bang(b_true.clone(), Some(&nil)).is_err());

        Ok(())
    }

    #[test]
    fn test_value_operation_equality_ok() -> Result<()> {
        let b_true = Value::Boolean(true);
        let b_false = Value::Boolean(false);
        let a_nubmer = Value::Number(6.0);
        let same_number = Value::Number(6.0);
        let different_number = Value::Number(7.0);
        let a_string = Value::String("hello".to_string());
        let same_string = Value::String("hello".to_string());
        let different_string = Value::String("world".to_string());
        let nil = Value::Nil;

        // region:    --- BOOL

        assert!(b_true.is_equal(&b_true));
        assert!(!b_true.is_equal(&b_false));
        assert!(!b_true.is_equal(&a_nubmer));
        assert!(!b_true.is_equal(&same_number));
        assert!(!b_true.is_equal(&different_number));
        assert!(!b_true.is_equal(&a_string));
        assert!(!b_true.is_equal(&same_string));
        assert!(!b_true.is_equal(&different_string));
        assert!(!b_true.is_equal(&nil));

        // endregion: --- BOOL

        // region:    --- NUMBER

        assert!(a_nubmer.is_equal(&a_nubmer));
        assert!(a_nubmer.is_equal(&same_number));
        assert!(!a_nubmer.is_equal(&different_number));
        assert!(!a_nubmer.is_equal(&b_true));
        assert!(!a_nubmer.is_equal(&b_false));
        assert!(!a_nubmer.is_equal(&a_string));
        assert!(!a_nubmer.is_equal(&same_string));
        assert!(!a_nubmer.is_equal(&different_string));
        assert!(!a_nubmer.is_equal(&nil));

        // endregion: --- NUMBER

        // region:    --- STRING

        assert!(a_string.is_equal(&a_string));
        assert!(a_string.is_equal(&same_string));
        assert!(!a_string.is_equal(&different_string));
        assert!(!a_string.is_equal(&b_true));
        assert!(!a_string.is_equal(&b_false));
        assert!(!a_string.is_equal(&a_nubmer));
        assert!(!a_string.is_equal(&same_number));
        assert!(!a_string.is_equal(&different_number));
        assert!(!a_string.is_equal(&nil));
        // endregion: --- STRING

        // region:    --- NIL

        assert!(nil.is_equal(&nil));
        assert!(!nil.is_equal(&b_true));
        assert!(!nil.is_equal(&b_false));
        assert!(!nil.is_equal(&a_nubmer));
        assert!(!nil.is_equal(&same_number));
        assert!(!nil.is_equal(&different_number));
        assert!(!nil.is_equal(&a_string));
        assert!(!nil.is_equal(&same_string));
        assert!(!nil.is_equal(&different_string));
        // endregion: --- NIL

        Ok(())
    }
}

// endregion: --- Tests
