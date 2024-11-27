use std::fmt::Debug;

use tracing_subscriber::fmt::format::Format;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    VAR,
    IDENTIFIER,
    EQUAL,
    STRING,
    SEMICOLON,
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: String,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: String, line: usize) -> Token {
        Token {
            token_type,
            lexeme,
            literal,
            line,
        }
    }

    pub fn eof(line: usize) -> Self {
        Token {
            token_type: TokenType::EOF,
            lexeme: String::new(),
            literal: String::from("null"),
            line,
        }
    }
}

impl core::fmt::Display for Token {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(
            fmt,
            "{:?} {} {}",
            self.token_type, self.lexeme, self.literal
        )
    }
}
