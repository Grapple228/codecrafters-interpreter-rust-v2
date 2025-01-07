mod error;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub use error::{Error, Result};

use crate::{Token, Value};

pub type MutEnv = Rc<RefCell<Environment>>;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Environment {
    values: HashMap<String, Option<Value>>,
    enclosing: Option<MutEnv>,
}

impl Environment {
    pub fn new(enclosing: Option<MutEnv>) -> Self {
        Environment {
            enclosing,
            ..Default::default()
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: Option<Value>) -> Result<()> {
        if let Some(ancestor) = self.ancestor(distance) {
            ancestor.borrow_mut().assign(name, value)?;
        }

        Ok(())
    }

    pub fn get_at(&self, distance: usize, name: &Token) -> Result<Value> {
        if let Some(ancestor) = self.ancestor(distance) {
            ancestor.borrow().get(&name)
        } else {
            Err(Error::AncestorNotFound(distance, name.clone()))
        }
    }

    fn ancestor(&self, distance: usize) -> Option<Rc<RefCell<Environment>>> {
        let mut env = Rc::new(RefCell::new(self.clone()));

        for _ in 0..distance {
            if let Some(enclosing) = &env.clone().borrow().enclosing {
                env = Rc::clone(enclosing);
            }
        }

        Some(env)
    }

    pub fn get(&self, name: &Token) -> Result<Value> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return if let Some(value) = value {
                Ok(value.clone())
            } else {
                Ok(Value::Nil)
            };
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name);
        }

        Err(Error::UndefinedVariable(name.to_owned()))
    }

    pub fn define(&mut self, name: &str, value: Option<Value>) {
        self.values.insert(name.to_string(), value);
    }

    pub fn assign(&mut self, name: &Token, value: Option<Value>) -> Result<()> {
        if let Some(existing) = self.values.get_mut(&name.lexeme) {
            *existing = value;

            return Ok(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            enclosing.borrow_mut().assign(name, value)?;
            return Ok(());
        }

        Err(Error::UndefinedVariable(name.clone()))
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

        assert_eq!(env.get(&token), Err(Error::UndefinedVariable(token)));

        Ok(())
    }

    #[test]
    fn test_variable_unitialized_ok() -> Result<()> {
        let mut env = Environment::default();

        let token = Token::new(TokenType::IDENTIFIER, "a", None, 1);

        env.define(&token.lexeme, None);

        assert_eq!(env.get(&token), Ok(Value::Nil));

        Ok(())
    }

    #[test]
    fn test_variable_initialized_ok() -> Result<()> {
        let mut env = Environment::default();

        let token = Token::new(TokenType::IDENTIFIER, "a", None, 1);
        let value = Value::Number(5.5);

        env.define(&token.lexeme, Some(value.clone()));

        assert_eq!(env.get(&token), Ok(value));

        Ok(())
    }

    #[test]
    fn test_variable_redefined_ok() -> Result<()> {
        let mut env = Environment::default();

        let token = Token::new(TokenType::IDENTIFIER, "a", None, 1);
        let value = Value::Number(5.5);

        env.define(&token.lexeme, Some(value.clone()));

        assert_eq!(env.get(&token), Ok(value));

        env.define(&token.lexeme, Some(Value::Number(6.6)));

        assert_eq!(env.get(&token), Ok(Value::Number(6.6)));

        Ok(())
    }
}

// endregion: --- Tests
