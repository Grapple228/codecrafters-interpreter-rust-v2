use tracing::debug;

use crate::{token::Value, visitor, Token, Visitor};

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
    pub fn accept(&self, visitor: &impl Visitor<String>) -> String {
        visitor.visit(self)
    }
}
