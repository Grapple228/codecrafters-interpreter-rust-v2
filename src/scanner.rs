use std::{fs, path::Path};

use tracing::{debug, info};

use crate::{Error, Result};
use crate::{StringExt, Token};

#[derive(Debug, Default)]
pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: usize,
    tokens: Vec<Token>,
}

impl Scanner {
    pub fn new(path: impl AsRef<Path>) -> Result<Scanner> {
        Ok(Scanner {
            source: fs::read_to_string(path)?,
            ..Default::default()
        })
    }

    fn is_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source.char_at(self.current);

        self.current += 1;

        c
    }

    fn scan_token(&mut self) {
        let c = self.advance();
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
}
