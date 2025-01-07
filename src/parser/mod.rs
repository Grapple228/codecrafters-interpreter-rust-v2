use tracing::info;

use crate::{tree::Expr, Stmt, Token, TokenType, Value};

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

    // region:    --- Statements

    pub fn parse_stmt(&mut self) -> Result<Vec<Stmt>> {
        info!("Parsing tokens into Stmt...");

        let mut stmts = Vec::new();

        while !self.is_end() {
            let stmt = self.declaration();

            match stmt {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    self.had_error = true;
                    Self::error(&e);
                    return Err(e);
                }
            }
        }

        Ok(stmts)
    }

    fn declaration(&mut self) -> Result<Stmt> {
        let stmt = if self.matches(&[TokenType::FUN]) {
            self.function("function")
        } else if self.matches(&[TokenType::VAR]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match stmt {
            Ok(stmt) => Ok(stmt),
            Err(e) => {
                self.synchronize();
                Err(e)
            }
        }
    }

    fn function(&mut self, kind: impl Into<String>) -> Result<Stmt> {
        let name = self.consume(TokenType::IDENTIFIER, "Expect function name.")?;

        self.consume(TokenType::LEFT_PAREN, "Expect '(' after function name.")?;

        let mut params = Vec::new();

        if !self.check(TokenType::RIGHT_PAREN) {
            loop {
                if params.len() >= 255 {
                    return Err(Error::TooManyArguments(self.peek()));
                }

                params.push(self.consume(TokenType::IDENTIFIER, "Expect parameter name.")?);

                if !self.matches(&[TokenType::COMMA]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after parameters.")?;

        self.consume(
            TokenType::LEFT_BRACE,
            format!("Expect '{{' before {} body.", kind.into()),
        )?;

        let body = self.block()?;

        Ok(Stmt::Function { name, params, body })
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(TokenType::IDENTIFIER, "Expect variable name.")?;

        let mut initializer = None;

        if self.matches(&[TokenType::EQUAL]) {
            initializer = Some(Box::new(self.expression()?));
        }

        self.consume(
            TokenType::SEMICOLON,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.matches(&[TokenType::FOR]) {
            return self.for_statement();
        }

        if self.matches(&[TokenType::IF]) {
            return self.if_statement();
        }

        if self.matches(&[TokenType::PRINT]) {
            return self.print_statement();
        }

        if self.matches(&[TokenType::RETURN]) {
            return self.return_statement();
        }

        if self.matches(&[TokenType::WHILE]) {
            return self.while_statement();
        }

        if self.matches(&[TokenType::LEFT_BRACE]) {
            return Ok(Stmt::Block(self.block()?));
        }

        self.expression_statement()
    }

    fn return_statement(&mut self) -> Result<Stmt> {
        let keyword = self.previous();
        let mut value = None;

        if !self.check(TokenType::SEMICOLON) {
            value = Some(Box::new(self.expression()?));
        }

        self.consume(TokenType::SEMICOLON, "Expect ';' after return value.")?;

        Ok(Stmt::Return { keyword, value })
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'for'.")?;

        let initializer = if self.matches(&[TokenType::SEMICOLON]) {
            None
        } else if self.matches(&[TokenType::VAR]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(TokenType::SEMICOLON) {
            self.expression()?
        } else {
            Expr::Literal(Some(Value::Boolean(true)))
        };

        self.consume(TokenType::SEMICOLON, "Expect ';' after loop condition.")?;

        let increment = if !self.check(TokenType::RIGHT_PAREN) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(Box::new(increment))]);
        }

        body = Stmt::While {
            condition: Box::new(condition),
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'while'.")?;
        let condition = self.expression();
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after condition.")?;

        let body = self.statement();

        Ok(Stmt::While {
            condition: Box::new(condition?),
            body: Box::new(body?),
        })
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'if'.")?;
        let condition = self.expression();
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after condition.")?;

        let then_branch = self.statement();

        let mut else_branch = None;
        if self.matches(&[TokenType::ELSE]) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If {
            condition: Box::new(condition?),
            then_branch: Box::new(then_branch?),
            else_branch,
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.check(TokenType::RIGHT_BRACE) && !self.is_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after block.")?;

        Ok(statements)
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
                Self::error(&e);
                Err(e)
            }
        }
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or();

        if self.matches(&[TokenType::EQUAL]) {
            let equals = self.previous();
            let value = self.assignment();

            if let Expr::Variable(name) = expr.clone()? {
                return Ok(Expr::Assign {
                    name: name.clone(),
                    value: Box::new(value?),
                });
            }

            Err(Error::InvalidAssignmentTarget(equals))?;
        }

        expr
    }

    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and();

        while self.matches(&[TokenType::OR]) {
            let operator = self.previous();
            let right = self.and();

            expr = Ok(Expr::Logical {
                left: Box::new(expr?),
                operator,
                right: Box::new(right?),
            });
        }

        expr
    }

    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality();

        while self.matches(&[TokenType::AND]) {
            let operator = self.previous();
            let right = self.equality();

            expr = Ok(Expr::Logical {
                left: Box::new(expr?),
                operator,
                right: Box::new(right?),
            });
        }

        expr
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

        self.call()
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary();

        loop {
            if self.matches(&[TokenType::LEFT_PAREN]) {
                expr = self.finish_call(expr?);
            } else {
                break;
            }
        }

        expr
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut arguments = Vec::new();

        if !self.check(TokenType::RIGHT_PAREN) {
            loop {
                if arguments.len() >= 255 {
                    return Err(Error::TooManyArguments(self.peek()));
                }

                arguments.push(self.expression()?);

                if !self.matches(&[TokenType::COMMA]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RIGHT_PAREN, "Expect ')' after arguments.")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
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
            return Ok(Expr::Literal(self.previous().literal));
        }

        if self.matches(&[TokenType::IDENTIFIER]) {
            return Ok(Expr::Variable(self.previous()));
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
        if self.check(token_type) {
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

    fn error(error: &Error) {
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
            Error::InvalidAssignmentTarget(token) => {
                crate::report(token.line, format!("Invalid assignment target."));
            }
            Error::TooManyArguments(token) => {
                crate::report(token.line, format!("Can't have more than 255 arguments."));
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
