//! Main Crate Error

use derive_more::derive::From;

use crate::{interpreter, parser};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    // -- Config
    ConfigMissingEnv(&'static str),
    ConfigWrongFormat(&'static str),

    UnknownCommand(String),
    ProgramExecutionError(String),

    // -- Modules
    #[from]
    ParserError(parser::Error),
    #[from]
    InterpreterError(interpreter::Error),

    // -- Externals
    #[from]
    IoError(std::io::Error),

    #[from]
    ParseFloatError(std::num::ParseFloatError),
}

// region:    --- Error Boilerplate

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
