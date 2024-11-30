use crate::{visitor::Acceptor, Visitor};

use super::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print(Box<Expr>),
    Expression(Box<Expr>),
}

// impl<T> Acceptor<T> for Stmt {
//     fn accept(&self, visitor: &impl Visitor<T>) -> T {
//         visitor.visit(self.clone())
//     }
// }
