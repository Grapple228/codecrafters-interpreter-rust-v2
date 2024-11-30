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
            Some(Some(value)) => Ok(value.clone()),
            _ => Err(Error::UndefinedVariable(name)),
        }
    }

    pub fn define(&mut self, name: String, value: Option<Value>) {
        self.values.insert(name, value);
    }
}
