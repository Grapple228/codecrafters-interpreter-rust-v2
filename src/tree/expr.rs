use tracing::debug;

use crate::{visitor, Token, Value, Visitor};

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
impl Expr {
    pub fn accept<T>(&self, visitor: &impl Visitor<T>) -> T {
        visitor.visit(self)
    }
}
