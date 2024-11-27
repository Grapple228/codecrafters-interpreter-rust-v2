use std::{fs, path::Path};

use tracing::{debug, info};

use crate::{Error, Result, SourceError, TokenType};
use crate::{StringExt, Token};

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

    fn error(&mut self, message: String) {
        self.errors.push(SourceError::Lexical(message, self.line));
    }

    fn is_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source.char_at(self.current);

        self.current += 1;

        c
    }

    fn peek(&mut self) -> char {
        if self.is_end() {
            return '\0';
        }

        self.source.char_at(self.current)
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
            '!' => {
                let token = if self.expect('=') {
                    TokenType::BANG_EQUAL
                } else {
                    TokenType::BANG
                };

                self.add_token(token)
            }
            '=' => {
                let token = if self.expect('=') {
                    TokenType::EQUAL_EQUAL
                } else {
                    TokenType::EQUAL
                };
                self.add_token(token)
            }
            '<' => {
                let token = if self.expect('=') {
                    TokenType::LESS_EQUAL
                } else {
                    TokenType::LESS
                };
                self.add_token(token)
            }
            '>' => {
                let token = if self.expect('=') {
                    TokenType::GREATER_EQUAL
                } else {
                    TokenType::GREATER
                };
                self.add_token(token)
            }
            '/' => {
                if self.expect('/') {
                    // A comment goes until the end of the line
                    while self.source.char_at(self.current) != '\n' && !self.is_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::SLASH)
                }
            }
            '\0' => {}
            ' ' => {}
            '\r' => {}
            '\t' => {}
            '\n' => {
                self.line += 1;
            }
            '"' => self.string(),

            _ => self.error(format!("Unexpected character: {}", c)),
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_end() {
            self.error("Unterminated string.".to_string());
            return;
        }

        // The closing quote
        self.advance();

        let value = self.source.substring(self.start + 1, self.current - 1);

        self.add_token_literal(TokenType::STRING, Some(value));
    }

    fn expect(&mut self, c: char) -> bool {
        if self.is_end() {
            return false;
        }

        if self.source.char_at(self.current) != c {
            return false;
        }

        self.current += 1;
        true
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
        // Fixtures
        let fx_content = "";
        let fx_tokens = vec![Token::eof(1)];

        // Init
        let mut scanner = Scanner::from_source(fx_content.to_string());

        scanner.scan_tokens()?;

        let tokens = scanner.tokens();

        // Check
        assert_eq!(tokens.len(), fx_tokens.len());
        assert_eq!(tokens, fx_tokens);

        Ok(())
    }

    #[test]
    fn test_parenthesis_ok() -> Result<()> {
        // Fixtures
        let fx_content = "(({{){})";
        let fx_tokens = vec![
            Token::new(TokenType::LEFT_PAREN, "(", None, 1),
            Token::new(TokenType::LEFT_PAREN, "(", None, 1),
            Token::new(TokenType::LEFT_BRACE, "{", None, 1),
            Token::new(TokenType::LEFT_BRACE, "{", None, 1),
            Token::new(TokenType::RIGHT_PAREN, ")", None, 1),
            Token::new(TokenType::LEFT_BRACE, "{", None, 1),
            Token::new(TokenType::RIGHT_BRACE, "}", None, 1),
            Token::new(TokenType::RIGHT_PAREN, ")", None, 1),
            Token::eof(1),
        ];

        // Init
        let mut scanner = Scanner::from_source(fx_content.to_string());

        scanner.scan_tokens()?;

        let tokens = scanner.tokens();

        // Check
        assert_eq!(tokens.len(), fx_tokens.len());
        assert_eq!(tokens, fx_tokens);

        Ok(())
    }

    #[test]
    fn test_error_std_ok() -> Result<()> {
        // Fixtures
        let fx_content = ",.$(#";
        let fx_errors = vec![
            SourceError::Lexical("Unexpected character: $".to_string(), 1),
            SourceError::Lexical("Unexpected character: #".to_string(), 1),
        ];

        let fx_tokens = vec![
            Token::new(TokenType::COMMA, ",", None, 1),
            Token::new(TokenType::DOT, ".", None, 1),
            Token::new(TokenType::LEFT_PAREN, "(", None, 1),
            Token::eof(1),
        ];

        // Init
        let mut scanner = Scanner::from_source(fx_content.to_string());

        scanner.scan_tokens()?;

        let tokens = scanner.tokens();
        let errors = scanner.errors();

        // Check
        assert_eq!(errors.len(), fx_errors.len());
        assert_eq!(tokens.len(), fx_tokens.len());

        assert_eq!(tokens, fx_tokens);
        assert_eq!(errors, fx_errors);

        Ok(())
    }

    #[test]
    fn test_double_symbol_operations_ok() -> Result<()> {
        // Fixtures
        let fx_content = "<<=>>=!!===";

        let fx_tokens = vec![
            Token::new(TokenType::LESS, "<", None, 1),
            Token::new(TokenType::LESS_EQUAL, "<=", None, 1),
            Token::new(TokenType::GREATER, ">", None, 1),
            Token::new(TokenType::GREATER_EQUAL, ">=", None, 1),
            Token::new(TokenType::BANG, "!", None, 1),
            Token::new(TokenType::BANG_EQUAL, "!=", None, 1),
            Token::new(TokenType::EQUAL_EQUAL, "==", None, 1),
            Token::eof(1),
        ];

        // Init
        let mut scanner = Scanner::from_source(fx_content.to_string());

        scanner.scan_tokens()?;

        let tokens = scanner.tokens();

        // Check
        assert_eq!(tokens.len(), fx_tokens.len());
        assert_eq!(tokens, fx_tokens);

        Ok(())
    }
}

// endregion: --- Tests
