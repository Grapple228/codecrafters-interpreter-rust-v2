use std::fmt::format;

use tracing::debug;

use crate::{visitor::Acceptor, Expr, Value, Visitor};

#[derive(Default, Clone)]
pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&self, expr: Expr) -> String {
        expr.accept(self)
    }
}

impl Visitor<String> for &AstPrinter {
    fn visit(&self, acceptor: impl Acceptor<String, Self>) -> String {
        acceptor.accept(&self)
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Error = Box<dyn std::error::Error>;
    type Result<T> = core::result::Result<T, Error>; // For tests.

    use crate::{Token, TokenType};

    use super::*;

    #[test]
    fn test_print_number_without_fraction_ok() -> Result<()> {
        // -- Setup & Fixtures
        let expr = Expr::Literal(Some(Value::Number(123.0)));

        // -- Exec
        let printer = AstPrinter::default();
        let result = printer.print(expr);

        // -- Check
        assert_eq!(result, "123.0");

        Ok(())
    }

    #[test]
    fn test_print_expr_ok() -> Result<()> {
        // -- Setup & Fixtures
        let expr = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token::new(TokenType::MINUS, "-", None, 1),
                right: Box::new(Expr::Literal(Some(Value::Number(123.0)))),
            }),
            operator: Token::new(TokenType::STAR, "*", None, 1),
            right: Box::new(Expr::Grouping(Box::new(Expr::Literal(Some(
                Value::Number(45.67),
            ))))),
        };

        // -- Exec
        let printer = AstPrinter::default();
        let result = printer.print(expr);

        // -- Check
        assert_eq!(result, "(* (- 123.0) (group 45.67))");

        Ok(())
    }
}

// endregion: --- Tests
