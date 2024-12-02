use crate::{Token, TokenType};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    UnknownExpression(Token),
    ExpectExpression(Token),
    UnexpectedToken(Token, String),
    InvalidAssignmentTarget(Token),
    TooManyArguments(Token),
}

// region:    --- Error Boilerplate

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
