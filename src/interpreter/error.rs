use std::sync::MutexGuard;

use derive_more::derive::From;

use crate::value;

use super::environment;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    ValueError(value::Error),
    #[from]
    EnvironmentError(environment::Error),

    // -- Externals
    #[from]
    MutexError(String),
}

// region:    --- Error Boilerplate

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
