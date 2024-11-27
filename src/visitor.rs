use crate::tree::Expr;

pub trait Visitor<T> {
    fn visit(&self, expr: &Expr) -> T;
}
