use crate::TokenType;

use super::Value;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    InvalidOperation {
        left: Value,
        right: Option<Value>,
        operator: TokenType,
    },
    InvalidType {
        left: Value,
        right: Option<Value>,
        operator: TokenType,
    },
    ZeroDivision {
        left: Value,
        right: Option<Value>,
    },
}

// region:    --- Error Boilerplate

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
