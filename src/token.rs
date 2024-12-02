use std::fmt::Debug;


use crate::Value;

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
    pub literal: Option<Value>,
    pub line: usize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: impl Into<String>,
        literal: Option<Value>,
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
            literal.to_string()
        } else {
            String::from("null")
        };

        write!(fmt, "{:?} {} {}", self.token_type, self.lexeme, literal)
    }
}

impl core::fmt::Display for TokenType {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        let op = match self {
            TokenType::LEFT_PAREN => "(",
            TokenType::RIGHT_PAREN => ")",
            TokenType::LEFT_BRACE => "{",
            TokenType::RIGHT_BRACE => "}",
            TokenType::COMMA => ",",
            TokenType::DOT => ".",
            TokenType::MINUS => "-",
            TokenType::PLUS => "+",
            TokenType::SEMICOLON => ";",
            TokenType::SLASH => "/",
            TokenType::STAR => "*",
            TokenType::BANG => "!",
            TokenType::BANG_EQUAL => "!=",
            TokenType::EQUAL => "=",
            TokenType::EQUAL_EQUAL => "==",
            TokenType::GREATER => ">",
            TokenType::GREATER_EQUAL => ">=",
            TokenType::LESS => "<",
            TokenType::LESS_EQUAL => "<=",
            TokenType::IDENTIFIER => "IDENTIFIER",
            TokenType::STRING => "STRING",
            TokenType::NUMBER => "NUMBER",
            TokenType::AND => "&",
            TokenType::CLASS => "CLASS",
            TokenType::ELSE => "ELSE",
            TokenType::FALSE => "FALSE",
            TokenType::FUN => "FUN",
            TokenType::FOR => "FOR",
            TokenType::IF => "IF",
            TokenType::NIL => "NIL",
            TokenType::OR => "OR",
            TokenType::PRINT => "PRINT",
            TokenType::RETURN => "RETURN",
            TokenType::SUPER => "SUPER",
            TokenType::THIS => "THIS",
            TokenType::TRUE => "TRUE",
            TokenType::VAR => "VAR",
            TokenType::WHILE => "WHILE",
            TokenType::EOF => "EOF",
        };

        write!(fmt, "{}", op)
    }
}
