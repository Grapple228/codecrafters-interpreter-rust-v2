use derive_more::derive::From;

use crate::{value, TokenType, Value};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, From)]
pub enum Error {
    #[from]
    ValueError(value::Error),
}

// region:    --- Error Boilerplate

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
