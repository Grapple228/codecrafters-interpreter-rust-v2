use tracing::{debug, info};

use crate::{tree::Expr, Stmt, Token, TokenType, Value, Visitor};

mod error;

pub use error::{Error, Result};

#[derive(Debug, Default)]
pub struct Parser {
    current: usize,
    tokens: Vec<Token>,
    had_error: bool,
}

impl Parser {
    pub fn new(tokens: &[Token]) -> Parser {
        Parser {
            tokens: tokens.to_vec(),
            ..Default::default()
        }
    }

    pub fn parse_stmt(&mut self) -> Result<Vec<Stmt>> {
        info!("Parsing tokens into Stmt...");

        let mut stmts = Vec::new();

        while !self.is_end() {
            let stmt = self.statement();

            match stmt {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    self.had_error = true;
                    Self::error(e.clone());
                    return Err(e);
                }
            }
        }

        Ok(stmts)
    }

    // region:    --- Statements

    fn statement(&mut self) -> Result<Stmt> {
        if self.matches(&[TokenType::PRINT]) {
            return self.print_statement();
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let value = self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.")?;
        Ok(Stmt::Print(Box::new(value?)))
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression();

        self.consume(TokenType::SEMICOLON, "Expect ';' after expression.")?;

        Ok(Stmt::Expression(Box::new(expr?)))
    }

    // endregion: --- Statements

    // region:    --- Expressions

    pub fn parse_expr(&mut self) -> Result<Expr> {
        info!("Parsing tokens into Expr...");
        let result = self.expression();

        match result {
            Ok(expr) => Ok(expr),
            Err(e) => {
                self.had_error = true;
                Self::error(e.clone());
                Err(e)
            }
        }
    }

    fn expression(&mut self) -> Result<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparsion();

        while self.matches(&[TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let operator = self.previous();
            let right = self.comparsion();

            expr = Ok(Expr::Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right?),
            });
        }

        expr
    }

    fn comparsion(&mut self) -> Result<Expr> {
        let mut expr = self.term();

        while self.matches(&[
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
        ]) {
            let operator = self.previous();
            let right = self.term();

            expr = Ok(Expr::Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right?),
            });
        }

        expr
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor();

        while self.matches(&[TokenType::MINUS, TokenType::PLUS]) {
            let operator = self.previous();
            let right = self.factor();

            expr = Ok(Expr::Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right?),
            });
        }

        expr
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary();

        while self.matches(&[TokenType::SLASH, TokenType::STAR]) {
            let operator = self.previous();
            let right = self.unary();

            expr = Ok(Expr::Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right?),
            });
        }

        expr
    }

    fn unary(&mut self) -> Result<Expr> {
        while self.matches(&[TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous();
            let right = self.unary();

            return Ok(Expr::Unary {
                operator,
                right: Box::new(right?),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.matches(&[TokenType::FALSE]) {
            return Ok(Expr::Literal(Some(Value::Boolean(false))));
        }
        if self.matches(&[TokenType::TRUE]) {
            return Ok(Expr::Literal(Some(Value::Boolean(true))));
        }
        if self.matches(&[TokenType::NIL]) {
            return Ok(Expr::Literal(Some(Value::Nil)));
        }

        if self.matches(&[TokenType::NUMBER, TokenType::STRING]) {
            let v = self.previous().literal;
            return Ok(Expr::Literal(self.previous().literal));
        }

        if self.matches(&[TokenType::LEFT_PAREN]) {
            let expr = self.expression();
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Box::new(expr?)));
        }

        Err(Error::ExpectExpression(self.peek()))?
    }

    // endregion: --- Expressions

    // region:    --- Helpers

    fn consume(&mut self, token_type: TokenType, message: impl Into<String>) -> Result<Token> {
        if self.check(token_type.clone()) {
            return Ok(self.advance());
        }

        Err(Error::UnexpectedToken(self.peek(), message.into()))?
    }

    fn synchronize(&mut self) -> () {
        self.advance();

        while !self.is_end() {
            {
                if self.previous().token_type == TokenType::SEMICOLON {
                    return;
                }

                match self.peek().token_type {
                    TokenType::CLASS
                    | TokenType::FUN
                    | TokenType::VAR
                    | TokenType::FOR
                    | TokenType::IF
                    | TokenType::WHILE
                    | TokenType::PRINT
                    | TokenType::RETURN => {
                        return;
                    }
                    _ => (),
                }

                self.advance();
            }
        }
    }

    fn is_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn advance(&mut self) -> Token {
        if !self.is_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn matches(&mut self, expected: &[TokenType]) -> bool {
        for token in expected {
            if self.check(token.clone()) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_end() {
            return false;
        }

        self.peek().token_type == token_type
    }

    // endregion: --- Helpers

    // region:    --- Error

    pub fn had_error(&self) -> bool {
        self.had_error
    }

    fn error(error: Error) {
        match error {
            Error::UnknownExpression(token) => {
                crate::report(token.line, "Unknown expression.");
            }
            Error::UnexpectedToken(token, message) => {
                crate::report(token.line, message);
            }
            Error::ExpectExpression(token) => {
                crate::report(token.line, format!("Expect expression."));
            }
        }
    }

    // endregion: --- Error
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Error = Box<dyn std::error::Error>;
    type Result<T> = core::result::Result<T, Error>; // For tests.

    use super::*;

    #[test]
    fn test_parse_nil_ok() -> Result<()> {
        // -- Setup & Fixtures
        let tokens = vec![Token::new(TokenType::NIL, "nil", None, 1), Token::eof(1)];

        // -- Exec
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse_expr()?;

        // -- Check
        assert_eq!(expr, Expr::Literal(Some(Value::Nil)));

        Ok(())
    }

    #[test]
    fn test_parse_true_ok() -> Result<()> {
        // -- Setup & Fixtures
        let tokens = vec![Token::new(TokenType::TRUE, "true", None, 1), Token::eof(1)];

        // -- Exec
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse_expr()?;

        // -- Check
        assert_eq!(expr, Expr::Literal(Some(Value::Boolean(true))));

        Ok(())
    }

    #[test]
    fn test_parse_false_ok() -> Result<()> {
        // -- Setup & Fixtures
        let tokens = vec![
            Token::new(TokenType::FALSE, "false", None, 1),
            Token::eof(1),
        ];

        // -- Exec
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse_expr()?;

        // -- Check
        assert_eq!(expr, Expr::Literal(Some(Value::Boolean(false))));

        Ok(())
    }
    #[test]
    fn test_parse_nubmer_sum_ok() -> Result<()> {
        // -- Setup & Fixtures
        let tokens = vec![
            Token::new(TokenType::NUMBER, "5.5", Some(Value::Number(5.5)), 1),
            Token::new(TokenType::PLUS, "+", None, 1),
            Token::new(TokenType::NUMBER, "6.6", Some(Value::Number(6.6)), 1),
            Token::eof(1),
        ];

        // -- Exec
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse_expr()?;

        // -- Check
        assert_eq!(
            expr,
            Expr::Binary {
                left: Box::new(Expr::Literal(Some(Value::Number(5.5)))),
                operator: Token::new(TokenType::PLUS, "+", None, 1),
                right: Box::new(Expr::Literal(Some(Value::Number(6.6)))),
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_nubmer_multiply_ok() -> Result<()> {
        // -- Setup & Fixtures
        let tokens = vec![
            Token::new(TokenType::NUMBER, "5.5", Some(Value::Number(5.5)), 1),
            Token::new(TokenType::PLUS, "*", None, 1),
            Token::new(TokenType::NUMBER, "6.6", Some(Value::Number(6.6)), 1),
            Token::eof(1),
        ];

        // -- Exec
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse_expr()?;

        // -- Check
        assert_eq!(
            expr,
            Expr::Binary {
                left: Box::new(Expr::Literal(Some(Value::Number(5.5)))),
                operator: Token::new(TokenType::PLUS, "*", None, 1),
                right: Box::new(Expr::Literal(Some(Value::Number(6.6)))),
            }
        );

        Ok(())
    }
}

// endregion: --- Tests
