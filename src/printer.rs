use tracing::debug;

use crate::{Expr, Value, Visitor};

#[derive(Default)]
pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&self, expr: Expr) -> String {
        expr.accept(self)
    }

    fn parenthesize(&self, name: impl Into<String>, exprs: &[&Box<Expr>]) -> String {
        let mut result = String::new();

        result.push('(');
        result.push_str(&name.into());

        for expr in exprs {
            result.push(' ');
            result.push_str(expr.accept(self).as_str());
        }

        result.push(')');

        result.to_string()
    }
}

impl Visitor<String> for AstPrinter {
    fn visit(&self, expr: &Expr) -> String {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.parenthesize(operator.lexeme.clone(), &[left, right]),
            Expr::Grouping(expr) => self.parenthesize("group", &[expr]),
            Expr::Literal(value) => match value {
                None => panic!("NONE IS IT RIGHT??"),
                Some(Value::String(s)) => s.clone(),
                Some(Value::Number(n)) => n.to_string(),
                Some(Value::Boolean(b)) => b.to_string(),
                Some(Value::Nil) => String::from("nil"),
            },
            Expr::Unary { operator, right } => self.parenthesize(operator.lexeme.clone(), &[right]),
        }
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
        assert_eq!(result, "(* (- 123) (group 45.67))");

        Ok(())
    }
}

// endregion: --- Tests
