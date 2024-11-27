use crate::{token::Value, tree::Expr, Token, TokenType, Visitor};

#[derive(Debug, Default)]
pub struct Parser {
    current: usize,
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: &[Token]) -> Parser {
        Parser {
            tokens: tokens.to_vec(),
            ..Default::default()
        }
    }

    pub fn parse(&mut self) -> Expr {
        //  TODO: Add error handling
        self.expression()
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparsion();

        while self.matches(&[TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let operator = self.previous();
            let right = self.comparsion();

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn comparsion(&mut self) -> Expr {
        let mut expr = self.term();

        while self.matches(&[
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
        ]) {
            let operator = self.previous();
            let right = self.term();

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.matches(&[TokenType::MINUS, TokenType::PLUS]) {
            let operator = self.previous();
            let right = self.factor();

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.matches(&[TokenType::SLASH, TokenType::STAR]) {
            let operator = self.previous();
            let right = self.unary();

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        while self.matches(&[TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous();
            let right = self.unary();

            return Expr::Unary {
                operator,
                right: Box::new(right),
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr {
        if self.matches(&[TokenType::FALSE]) {
            return Expr::Literal(Some(Value::Boolean(false)));
        }
        if self.matches(&[TokenType::TRUE]) {
            return Expr::Literal(Some(Value::Boolean(true)));
        }
        if self.matches(&[TokenType::NIL]) {
            return Expr::Literal(Some(Value::Nil));
        }

        if self.matches(&[TokenType::NUMBER, TokenType::STRING]) {
            let v = self.previous().literal;
            return Expr::Literal(self.previous().literal);
        }

        if self.matches(&[TokenType::LEFT_PAREN]) {
            let expr = self.expression();
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.");
            return Expr::Grouping(Box::new(expr));
        }

        panic!("{} Expect expression", self.peek());
    }

    fn consume(&mut self, token_type: TokenType, message: impl Into<String>) -> Token {
        if self.check(token_type) {
            return self.advance();
        }

        // TODO: Add error handling
        panic!("{}", message.into());
    }

    fn error() {
        todo!()
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
        self.current >= self.tokens.len()
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
}
