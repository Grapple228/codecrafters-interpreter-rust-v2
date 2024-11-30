mod error;

use std::{collections::HashMap, hash::Hash};

pub use error::{Error, Result};

use crate::{Token, Value};

#[derive(Debug, Clone, Default)]
pub struct Environment {
    values: HashMap<String, Option<Value>>,
}

impl Environment {
    pub fn get(&self, name: Token) -> Result<Value> {
        match self.values.get(&name.lexeme) {
            Some(value) => {
                if let Some(value) = value {
                    Ok(value.clone())
                } else {
                    Ok(Value::Nil)
                }
            }
            _ => Err(Error::UndefinedVariable(name)),
        }
    }

    pub fn define(&mut self, name: String, value: Option<Value>) {
        self.values.insert(name, value);
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T> = core::result::Result<T, Error>; // For tests.

    use crate::TokenType;

    use super::*;

    #[test]
    fn test_variable_undefined_err() -> Result<()> {
        let env = Environment::default();

        let token = Token::new(TokenType::IDENTIFIER, "a", None, 1);

        assert_eq!(env.get(token.clone()), Err(Error::UndefinedVariable(token)));

        Ok(())
    }

    #[test]
    fn test_variable_unitialized_ok() -> Result<()> {
        let mut env = Environment::default();

        let token = Token::new(TokenType::IDENTIFIER, "a", None, 1);

        env.define(token.lexeme.clone(), None);

        assert_eq!(env.get(token.clone()), Ok(Value::Nil));

        Ok(())
    }

    #[test]
    fn test_variable_initialized_ok() -> Result<()> {
        let mut env = Environment::default();

        let token = Token::new(TokenType::IDENTIFIER, "a", None, 1);
        let value = Value::Number(5.5);

        env.define(token.lexeme.clone(), Some(value.clone()));

        assert_eq!(env.get(token.clone()), Ok(value));

        Ok(())
    }

    #[test]
    fn test_variable_redefined_ok() -> Result<()> {
        let mut env = Environment::default();

        let token = Token::new(TokenType::IDENTIFIER, "a", None, 1);
        let value = Value::Number(5.5);

        env.define(token.lexeme.clone(), Some(value.clone()));

        assert_eq!(env.get(token.clone()), Ok(value));

        env.define(token.lexeme.clone(), Some(Value::Number(6.6)));

        assert_eq!(env.get(token.clone()), Ok(Value::Number(6.6)));

        Ok(())
    }
}

// endregion: --- Tests
