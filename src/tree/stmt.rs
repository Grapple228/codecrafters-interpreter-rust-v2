use crate::{visitor::Acceptor, AstPrinter, Interpreter, Visitor};

use super::Expr;
use crate::interpreter::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print(Box<Expr>),
    Expression(Box<Expr>),
}

impl Acceptor<Result<()>, &Interpreter> for Stmt {
    fn accept(&self, visitor: &Interpreter) -> Result<()> {
        match self {
            Stmt::Expression(expr) => {
                let _ = expr.accept(visitor);
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = expr.accept(visitor)?;
                println!("{}", value.stringify());
                Ok(())
            }
        }
    }
}

impl Acceptor<String, &AstPrinter> for Stmt {
    fn accept(&self, visitor: &AstPrinter) -> String {
        match self {
            Stmt::Expression(expr) => expr.accept(visitor),
            Stmt::Print(expr) => expr.accept(visitor),
        }
    }
}
