use std::fmt::Debug;

use tracing_subscriber::fmt::format::Format;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // One or two character tokens.
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,

    // Literals.
    IDENTIFIER,
    STRING,
    NUMBER,

    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<String>,
    pub line: usize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: impl Into<String>,
        literal: Option<String>,
        line: usize,
    ) -> Token {
        Token {
            token_type,
            lexeme: lexeme.into(),
            literal,
            line,
        }
    }

    pub fn eof(line: usize) -> Self {
        Token {
            token_type: TokenType::EOF,
            lexeme: String::new(),
            literal: None,
            line,
        }
    }
}

impl core::fmt::Display for Token {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        let literal = if let Some(literal) = self.literal.clone() {
            literal
        } else {
            String::from("null")
        };

        write!(fmt, "{:?} {} {}", self.token_type, self.lexeme, literal)
    }
}
