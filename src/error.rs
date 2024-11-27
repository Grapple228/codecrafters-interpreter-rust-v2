//! Main Crate Error

use derive_more::derive::From;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum SourceError {
    Lexical(String, usize),
    Runtime(String, usize),
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SourceError::Lexical(message, line) => {
                write!(fmt, "[line {}] Error: {}", line, message)
            }
            SourceError::Runtime(_, _) => todo!(),
        }
    }
}

#[derive(Debug, From)]
pub enum Error {
    // -- Config
    ConfigMissingEnv(&'static str),
    ConfigWrongFormat(&'static str),

    UnknownCommand(String),
    ProgramExecutionError(String),

    // -- Modules

    // -- Externals
    #[from]
    IoError(std::io::Error),
}

// region:    --- Error Boilerplate

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
