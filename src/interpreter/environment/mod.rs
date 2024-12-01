mod error;

use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Mutex},
};

pub use error::{Error, Result};

use crate::{Token, Value, W};

#[derive(Debug, Clone, Default)]
pub struct Environment {
    values: HashMap<String, Option<Value>>,
    enclosing: Option<Arc<Mutex<Environment>>>,
}

impl Environment {
    pub fn new(enclosing: Option<Arc<Mutex<Environment>>>) -> Self {
        Environment {
            enclosing,
            ..Default::default()
        }
    }

    pub fn get(&self, name: Token) -> Result<Value> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return if let Some(value) = value {
                Ok(value.clone())
            } else {
                Ok(Value::Nil)
            };
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing
                .lock()
                .map_err(|e| Error::MutexError(name.clone(), e.to_string()))?
                .get(name);
        }

        Err(Error::UndefinedVariable(name))
    }

    pub fn define(&mut self, name: Token, value: Option<Value>) {
        self.values.insert(name.lexeme.clone(), value);
    }

    pub fn assign(&mut self, name: Token, value: Option<Value>) -> Result<()> {
        if let Some(value) = self.values.get_mut(&name.lexeme) {
            *value = value.clone();
            return Ok(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            enclosing
                .lock()
                .map_err(|e| Error::MutexError(name.clone(), e.to_string()))?
                .assign(name, value)?;
            return Ok(());
        }

        Err(Error::UndefinedVariable(name))
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

        env.define(token.clone(), None);

        assert_eq!(env.get(token.clone()), Ok(Value::Nil));

        Ok(())
    }

    #[test]
    fn test_variable_initialized_ok() -> Result<()> {
        let mut env = Environment::default();

        let token = Token::new(TokenType::IDENTIFIER, "a", None, 1);
        let value = Value::Number(5.5);

        env.define(token.clone(), Some(value.clone()));

        assert_eq!(env.get(token.clone()), Ok(value));

        Ok(())
    }

    #[test]
    fn test_variable_redefined_ok() -> Result<()> {
        let mut env = Environment::default();

        let token = Token::new(TokenType::IDENTIFIER, "a", None, 1);
        let value = Value::Number(5.5);

        env.define(token.clone(), Some(value.clone()));

        assert_eq!(env.get(token.clone()), Ok(value));

        env.define(token.clone(), Some(Value::Number(6.6)));

        assert_eq!(env.get(token.clone()), Ok(Value::Number(6.6)));

        Ok(())
    }
}

// endregion: --- Tests
