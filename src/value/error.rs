use crate::{Token, TokenType};

use super::Value;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidOperation {
        left: Value,
        right: Option<Value>,
        token: Token,
        message: String,
    },
    InvalidType {
        left: Value,
        right: Option<Value>,
        token: Token,
        message: String,
    },
    ZeroDivision {
        left: Value,
        right: Option<Value>,
        token: Token,
        message: String,
    },
    MustBeNumber {
        left: Value,
        token: Token,
        right: Option<Value>,
        message: String,
    },
    MustBeNumberOrString {
        left: Value,
        token: Token,
        right: Option<Value>,
        message: String,
    },
    NotCallable {
        token: Token,
    },
    InvalidCountOfArguments {
        token: Token,
        count: usize,
        expected: usize,
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
