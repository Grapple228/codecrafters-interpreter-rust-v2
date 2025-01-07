use std::cell::RefCell;
use std::rc::Rc;

use crate::interpreter::{self, Environment};
use crate::resolver::{self, FunctionType, MutResolver, Resolver};
use crate::{visitor::Acceptor, AstPrinter, Token};
use crate::{Callable, MutInterpreter, Value};

use super::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print(Box<Expr>),
    Expression(Box<Expr>),
    Var {
        name: Token,
        initializer: Option<Box<Expr>>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Return {
        keyword: Token,
        value: Option<Box<Expr>>,
    },
}

impl Acceptor<resolver::Result<()>, &MutResolver> for Stmt {
    fn accept(&self, visitor: &MutResolver) -> resolver::Result<()> {
        match self {
            Stmt::Block(stmts) => {
                visitor.borrow_mut().begin_scope();

                Resolver::resolve_block(visitor, &stmts)?;

                visitor.borrow_mut().end_scope();

                Ok(())
            }
            Stmt::Var { name, initializer } => {
                visitor.borrow_mut().declare(&name)?;

                if let Some(initializer) = initializer {
                    initializer.accept(visitor)?;
                }

                visitor.borrow_mut().define(&name);

                Ok(())
            }
            Stmt::Function { name, params, body } => {
                visitor.borrow_mut().declare(&name)?;
                visitor.borrow_mut().define(&name);

                let enclosing_function = visitor
                    .borrow_mut()
                    .replace_function(resolver::FunctionType::Function);

                visitor.borrow_mut().begin_scope();

                for param in params {
                    visitor.borrow_mut().declare(&param)?;
                    visitor.borrow_mut().define(&param);
                }

                Resolver::resolve_block(visitor, &body)?;

                visitor.borrow_mut().end_scope();

                _ = visitor.borrow_mut().replace_function(enclosing_function);

                Ok(())
            }
            Stmt::Expression(expr) => {
                expr.accept(visitor)?;
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                condition.accept(visitor)?;
                then_branch.accept(visitor)?;

                if let Some(else_branch) = else_branch {
                    else_branch.accept(visitor)?;
                }

                Ok(())
            }
            Stmt::Print(expr) => {
                expr.accept(visitor)?;
                Ok(())
            }

            Stmt::Return { keyword, value } => {
                if visitor.borrow().current_function() == FunctionType::None {
                    return Err(resolver::Error::TopLevelReturn(keyword.clone()));
                }

                if let Some(value) = value {
                    value.accept(visitor)?;
                }

                Ok(())
            }
            Stmt::While { condition, body } => {
                condition.accept(visitor)?;
                body.accept(visitor)?;

                Ok(())
            }
        }
    }
}

impl Acceptor<interpreter::Result<()>, &MutInterpreter> for Stmt {
    fn accept(&self, visitor: &MutInterpreter) -> interpreter::Result<()> {
        match self {
            Stmt::Expression(expr) => {
                let _ = expr.accept(visitor)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = expr.accept(visitor)?;
                println!("{}", value.stringify());
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let mut value = None;

                if let Some(initializer) = initializer {
                    value = Some(initializer.accept(visitor)?);
                };

                let interpreter = visitor.borrow();

                interpreter
                    .environment
                    .borrow_mut()
                    .define(&name.lexeme, value);

                Ok(())
            }
            Stmt::Block(stmts) => {
                let mut interpreter = visitor.borrow_mut();

                let env = Environment::new(Some(interpreter.environment.clone()));
                interpreter.execute_block(stmts, Rc::new(RefCell::new(env)))
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let value = condition.accept(visitor)?;

                if value.is_truthy() {
                    then_branch.accept(visitor)
                } else if let Some(else_branch) = else_branch {
                    else_branch.accept(visitor)
                } else {
                    Ok(())
                }
            }
            Stmt::While { condition, body } => {
                while condition.accept(visitor)?.is_truthy() {
                    body.accept(visitor)?
                }

                Ok(())
            }
            Stmt::Function { name, params, body } => {
                let interpreter = visitor.borrow();

                let value = Value::Callable(Callable::Function {
                    declaration: Box::new(Stmt::Function {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                    }),
                    closure: interpreter.environment.clone(),
                });

                interpreter
                    .environment
                    .borrow_mut()
                    .define(&name.lexeme, Some(value));

                Ok(())
            }
            Stmt::Return { value, .. } => {
                let mut result = Value::Nil;

                if let Some(value) = value {
                    result = value.accept(visitor)?;
                }

                Err(interpreter::Error::Return(result))?
            }
        }
    }
}

impl Acceptor<String, &AstPrinter> for Stmt {
    fn accept(&self, visitor: &AstPrinter) -> String {
        match self {
            Stmt::Expression(expr) => expr.accept(visitor),
            Stmt::Print(expr) => {
                format!("print {}", expr.accept(visitor))
            }
            Stmt::Var { name, initializer } => {
                let mut result = String::new();

                result.push_str("var ");
                result.push_str(&name.lexeme);

                if let Some(initializer) = initializer {
                    result.push_str(" = ");
                    result.push_str(&initializer.accept(visitor));
                }

                result
            }
            Stmt::Block(stmts) => {
                let mut result = String::new();

                result.push_str("{\n");

                for stmt in stmts {
                    result.push_str(&stmt.accept(visitor));
                    result.push_str("\n");
                }

                result.push_str("}\n");

                result
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut result = String::new();

                result.push_str("if (");
                result.push_str(&condition.accept(visitor));
                result.push_str(") {");
                result.push_str(&then_branch.accept(visitor));
                result.push_str("}");

                if let Some(else_branch) = else_branch {
                    result.push_str(" else {");
                    result.push_str(&else_branch.accept(visitor));
                    result.push_str("}");
                }

                result
            }
            Stmt::While { condition, body } => {
                let mut result = String::new();

                result.push_str("while ");
                result.push_str(&condition.accept(visitor));
                result.push_str(&body.accept(visitor));

                result
            }
            Stmt::Function { name, params, body } => {
                let mut result = String::new();

                result.push_str("fn ");
                result.push_str(&name.lexeme);

                result.push_str("(");
                result.push_str(
                    &params
                        .iter()
                        .map(|p| p.lexeme.clone())
                        .collect::<Vec<String>>()
                        .join(", "),
                );
                result.push_str(") {");
                for b in body {
                    result.push_str(&b.accept(visitor));
                }
                result.push_str("}");

                result
            }
            Stmt::Return { value, .. } => {
                let mut result = String::new();

                if let Some(value) = value {
                    result.push_str("return ");
                    result.push_str(&value.accept(visitor));
                } else {
                    result.push_str("return");
                }

                result
            }
        }
    }
}
