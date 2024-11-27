use std::{fs, path::Path};

use tracing::{debug, info};

use crate::{Error, Result, TokenType};
use crate::{StringExt, Token};

#[derive(Debug, Clone)]
pub enum SourceError {
    RuntimeError(String, usize),
    CompileError(String, usize),
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SourceError::CompileError(content, line) => {
                write!(fmt, "[line {}] Error: {}", line, content)
            }
            SourceError::RuntimeError(content, line) => {
                write!(fmt, "[line {}] Runtime Error: {}", line, content)
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: usize,
    tokens: Vec<Token>,
    errors: Vec<SourceError>,
}

impl Scanner {
    /// Create a new scanner from source
    pub fn from_source(source: impl Into<String>) -> Scanner {
        Scanner {
            source: source.into(),
            line: 1,
            ..Default::default()
        }
    }

    /// Create a new scanner from a file
    pub fn new(path: impl AsRef<Path>) -> Result<Scanner> {
        Ok(Scanner {
            source: fs::read_to_string(path)?,
            line: 1,
            ..Default::default()
        })
    }

    pub fn has_error(&self) -> bool {
        self.errors.len() != 0
    }

    fn error(&mut self, message: impl Into<String>) {
        self.errors
            .push(SourceError::CompileError(message.into(), self.line));
    }

    fn is_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source.char_at(self.current);

        self.current += 1;

        c
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_literal(token_type, None)
    }

    fn add_token_literal(&mut self, token_type: TokenType, literal: Option<String>) {
        let lexeme = self.source.substring(self.start, self.current);

        self.tokens
            .push(Token::new(token_type, lexeme, literal, self.line));
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LEFT_PAREN),
            ')' => self.add_token(TokenType::RIGHT_PAREN),
            '{' => self.add_token(TokenType::LEFT_BRACE),
            '}' => self.add_token(TokenType::RIGHT_BRACE),
            ',' => self.add_token(TokenType::COMMA),
            '.' => self.add_token(TokenType::DOT),
            '-' => self.add_token(TokenType::MINUS),
            '+' => self.add_token(TokenType::PLUS),
            ';' => self.add_token(TokenType::SEMICOLON),
            '*' => self.add_token(TokenType::STAR),

            _ => self.error(format!("Unexpected character: {}", c)),
        }
    }

    pub fn scan_tokens(&mut self) -> Result<()> {
        info!("Scanning tokens...");

        while !self.is_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::eof(self.line));

        Ok(())
    }

    pub fn tokens(&self) -> Vec<Token> {
        self.tokens.clone()
    }

    pub fn errors(&self) -> Vec<SourceError> {
        self.errors.clone()
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_empty_file_ok() -> Result<()> {
        let fx_content = "";

        let mut scanner = Scanner::from_source(fx_content.to_string());

        scanner.scan_tokens()?;

        let tokens = scanner.tokens();

        assert_eq!(tokens.len(), 1);

        assert_eq!(tokens, vec![Token::eof(0)]);

        Ok(())
    }

    #[test]
    fn test_parenthesis_ok() -> Result<()> {
        let fx_content = "(({{){})";
        let fx_tokens = vec![
            Token::new(TokenType::LEFT_PAREN, "(", None, 0),
            Token::new(TokenType::LEFT_PAREN, "(", None, 0),
            Token::new(TokenType::LEFT_BRACE, "{", None, 0),
            Token::new(TokenType::LEFT_BRACE, "{", None, 0),
            Token::new(TokenType::RIGHT_PAREN, ")", None, 0),
            Token::new(TokenType::LEFT_BRACE, "{", None, 0),
            Token::new(TokenType::RIGHT_BRACE, "}", None, 0),
            Token::new(TokenType::RIGHT_PAREN, ")", None, 0),
            Token::eof(0),
        ];

        let mut scanner = Scanner::from_source(fx_content.to_string());

        scanner.scan_tokens()?;

        let tokens = scanner.tokens();

        assert_eq!(tokens.len(), 9);

        assert_eq!(tokens, fx_tokens);

        Ok(())
    }
}

// endregion: --- Tests
