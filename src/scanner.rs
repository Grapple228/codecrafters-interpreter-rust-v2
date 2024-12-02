use std::collections::HashMap;
use std::{fs, path::Path};

use tracing::info;

use crate::extensions::{CharExt, StringExt};
use crate::Token;
use crate::Value;
use crate::{report, Result, TokenType};
use lazy_static::lazy_static;

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenType> = {
        let mut hm = HashMap::new();

        hm.insert("and", TokenType::AND);
        hm.insert("class", TokenType::CLASS);
        hm.insert("else", TokenType::ELSE);
        hm.insert("false", TokenType::FALSE);
        hm.insert("for", TokenType::FOR);
        hm.insert("fun", TokenType::FUN);
        hm.insert("if", TokenType::IF);
        hm.insert("nil", TokenType::NIL);
        hm.insert("or", TokenType::OR);
        hm.insert("print", TokenType::PRINT);
        hm.insert("return", TokenType::RETURN);
        hm.insert("super", TokenType::SUPER);
        hm.insert("this", TokenType::THIS);
        hm.insert("true", TokenType::TRUE);
        hm.insert("var", TokenType::VAR);
        hm.insert("while", TokenType::WHILE);

        hm
    };
}

#[derive(Debug, Default)]
pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: usize,
    tokens: Vec<Token>,
    had_error: bool,
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

    pub fn had_error(&self) -> bool {
        self.had_error
    }

    fn error(&mut self, message: String) {
        self.had_error = true;
        report(self.line, message);
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

    fn peek_next(&mut self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        self.source.char_at(self.current + 1)
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_literal(token_type, None)
    }

    fn add_token_literal(&mut self, token_type: TokenType, literal: Option<Value>) {
        let lexeme = self.source.substring(self.start, self.current);

        self.tokens
            .push(Token::new(token_type, lexeme, literal, self.line));
    }

    fn scan_token(&mut self) -> Result<()> {
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

            other => {
                if other.is_ascii_digit() {
                    self.number()?;
                } else if other.is_alpha() {
                    self.identifier();
                } else {
                    self.error(format!("Unexpected character: {}", c))
                }
            }
        }

        Ok(())
    }

    fn identifier(&mut self) {
        while self.peek().is_alpha_numeric() {
            self.advance();
        }

        let lexeme = self.source.substring(self.start, self.current);
        let token_type = KEYWORDS
            .get(lexeme.as_str())
            .cloned()
            .unwrap_or(TokenType::IDENTIFIER);

        self.add_token(token_type);
    }

    fn number(&mut self) -> Result<()> {
        while self.peek().is_digit(10) {
            self.advance();
        }

        // Look for a fractional part
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            // Consume the "."
            self.advance();

            while self.peek().is_digit(10) {
                self.advance();
            }
        };

        let value = self.source.substring(self.start, self.current);

        self.add_token_literal(TokenType::NUMBER, Some(Value::Number(value.parse()?)));

        Ok(())
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

        self.add_token_literal(TokenType::STRING, Some(Value::String(value)));
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
            let _ = self.scan_token();
        }

        self.tokens.push(Token::eof(self.line));

        Ok(())
    }

    pub fn tokens(&self) -> Vec<Token> {
        self.tokens.clone()
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
        let fx_tokens = vec!["EOF  null"];

        // Init
        let mut scanner = Scanner::from_source(fx_content.to_string());

        scanner.scan_tokens()?;

        let tokens = scanner.tokens();

        // Check
        assert_eq!(tokens.len(), fx_tokens.len());
        assert_eq!(
            tokens
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>(),
            fx_tokens
        );

        Ok(())
    }

    #[test]
    fn test_identifier_ok() -> Result<()> {
        // Fixtures
        let fx_content = "fo3o ba2r _hello_";
        let fx_tokens = vec![
            "IDENTIFIER fo3o null",
            "IDENTIFIER ba2r null",
            "IDENTIFIER _hello_ null",
            "EOF  null",
        ];

        // Init
        let mut scanner = Scanner::from_source(fx_content.to_string());

        scanner.scan_tokens()?;

        let tokens = scanner.tokens();

        // Check
        assert_eq!(tokens.len(), fx_tokens.len());
        assert_eq!(
            tokens
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>(),
            fx_tokens
        );

        Ok(())
    }

    #[test]
    fn test_reserved_ok() -> Result<()> {
        // Fixtures
        let fx_content = "fun class _hello_";
        let fx_tokens = vec![
            "FUN fun null",
            "CLASS class null",
            "IDENTIFIER _hello_ null",
            "EOF  null",
        ];

        // Init
        let mut scanner = Scanner::from_source(fx_content.to_string());

        scanner.scan_tokens()?;

        let tokens = scanner.tokens();

        // Check
        assert_eq!(tokens.len(), fx_tokens.len());
        assert_eq!(
            tokens
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>(),
            fx_tokens
        );

        Ok(())
    }

    #[test]
    fn test_comment_ok() -> Result<()> {
        // Fixtures
        let fx_content = "// Hello\n42";
        let fx_tokens = vec!["NUMBER 42 42.0", "EOF  null"];

        // Init
        let mut scanner = Scanner::from_source(fx_content.to_string());

        scanner.scan_tokens()?;

        let tokens = scanner.tokens();

        // Check
        assert_eq!(tokens.len(), fx_tokens.len());
        assert_eq!(
            tokens
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>(),
            fx_tokens
        );

        Ok(())
    }

    #[test]
    fn test_number_ok() -> Result<()> {
        // Fixtures
        let fx_content = "42";
        let fx_tokens = vec!["NUMBER 42 42.0", "EOF  null"];

        // Init
        let mut scanner = Scanner::from_source(fx_content.to_string());

        scanner.scan_tokens()?;

        let tokens = scanner.tokens();

        // Check
        assert_eq!(tokens.len(), fx_tokens.len());
        assert_eq!(
            tokens
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>(),
            fx_tokens
        );

        Ok(())
    }

    #[test]
    fn test_string_ok() -> Result<()> {
        // Fixtures
        let fx_content = "\"foo\"";
        let fx_tokens = vec!["STRING \"foo\" foo", "EOF  null"];

        // Init
        let mut scanner = Scanner::from_source(fx_content.to_string());

        scanner.scan_tokens()?;

        let tokens = scanner.tokens();

        // Check
        assert_eq!(tokens.len(), fx_tokens.len());
        assert_eq!(
            tokens
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>(),
            fx_tokens
        );

        Ok(())
    }

    #[test]
    fn test_parenthesis_ok() -> Result<()> {
        // Fixtures
        let fx_content = "(({{){})";
        let fx_tokens = vec![
            "LEFT_PAREN ( null",
            "LEFT_PAREN ( null",
            "LEFT_BRACE { null",
            "LEFT_BRACE { null",
            "RIGHT_PAREN ) null",
            "LEFT_BRACE { null",
            "RIGHT_BRACE } null",
            "RIGHT_PAREN ) null",
            "EOF  null",
        ];

        // Init
        let mut scanner = Scanner::from_source(fx_content.to_string());

        scanner.scan_tokens()?;

        let tokens = scanner.tokens();

        // Check
        assert_eq!(tokens.len(), fx_tokens.len());
        assert_eq!(
            tokens
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>(),
            fx_tokens
        );

        Ok(())
    }

    #[test]
    fn test_error_ok() -> Result<()> {
        // Fixtures
        let fx_content = ",.$(#";

        let fx_tokens = vec![
            "COMMA , null",
            "DOT . null",
            "LEFT_PAREN ( null",
            "EOF  null",
        ];

        // Init
        let mut scanner = Scanner::from_source(fx_content.to_string());

        scanner.scan_tokens()?;

        let tokens = scanner.tokens();

        // Check
        assert!(scanner.had_error);
        assert_eq!(tokens.len(), fx_tokens.len());

        assert_eq!(
            tokens
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>(),
            fx_tokens
        );

        Ok(())
    }

    #[test]
    fn test_double_symbol_operations_ok() -> Result<()> {
        // Fixtures
        let fx_content = "<<=>>=!!===";

        let fx_tokens = vec![
            "LESS < null",
            "LESS_EQUAL <= null",
            "GREATER > null",
            "GREATER_EQUAL >= null",
            "BANG ! null",
            "BANG_EQUAL != null",
            "EQUAL_EQUAL == null",
            "EOF  null",
        ];

        // Init
        let mut scanner = Scanner::from_source(fx_content.to_string());

        scanner.scan_tokens()?;

        let tokens = scanner.tokens();

        // Check
        assert_eq!(tokens.len(), fx_tokens.len());
        assert_eq!(
            tokens
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>(),
            fx_tokens
        );

        Ok(())
    }
}

// endregion: --- Tests
