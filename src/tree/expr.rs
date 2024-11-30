use tracing::debug;

use crate::interpreter::Result;
use crate::{
    visitor::{self, Acceptor},
    AstPrinter, Interpreter, Token, Value, Visitor,
};

use super::Stmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Option<Value>),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

impl Into<Stmt> for Expr {
    fn into(self) -> Stmt {
        Stmt::Expression(Box::new(self))
    }
}

impl Expr {
    fn parenthesize(visitor: &AstPrinter, name: impl Into<String>, exprs: &[&Box<Expr>]) -> String {
        let mut result = String::new();

        result.push('(');
        result.push_str(&name.into());

        for expr in exprs {
            result.push(' ');
            result.push_str(expr.accept(visitor).as_str());
        }

        result.push(')');

        result.to_string()
    }
}

impl Acceptor<Result<Value>, &Interpreter> for Expr {
    fn accept(&self, visitor: &Interpreter) -> Result<Value> {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = left.accept(&visitor.clone())?;
                let right = right.accept(visitor)?;

                Ok(left.calculate(Some(&right), operator.clone())?)
            }
            Expr::Grouping(expr) => expr.accept(visitor),
            Expr::Literal(value) => {
                if let Some(value) = value.clone() {
                    Ok(value)
                } else {
                    Ok(Value::Nil)
                }
            }
            Expr::Unary { operator, right } => {
                let value = right.accept(visitor)?;

                Ok(value.calculate(None, operator.clone())?)
            }
        }
    }
}

impl Acceptor<String, &AstPrinter> for Expr {
    fn accept(&self, visitor: &AstPrinter) -> String {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => Self::parenthesize(&visitor, operator.lexeme.clone(), &[left, right]),
            Expr::Grouping(expr) => Self::parenthesize(&visitor, "group", &[expr]),
            Expr::Literal(value) => match value {
                None => panic!("NONE IS IT RIGHT??"),
                Some(Value::String(s)) => s.clone(),
                Some(Value::Number(n)) => format!("{:?}", n),
                Some(Value::Boolean(b)) => b.to_string(),
                Some(Value::Nil) => String::from("nil"),
            },
            Expr::Unary { operator, right } => {
                Self::parenthesize(&visitor, operator.lexeme.clone(), &[right])
            }
        }
    }
}
