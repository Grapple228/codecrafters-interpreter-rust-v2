use crate::Token;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidOperation {
        token: Token,
        message: String,
    },
    InvalidType {
        token: Token,
        message: String,
    },
    ZeroDivision {
        token: Token,
        message: String,
    },
    MustBeNumber {
        token: Token,
        message: String,
    },
    MustBeNumberOrString {
        token: Token,
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
