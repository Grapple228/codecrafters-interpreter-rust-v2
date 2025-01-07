mod error;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub use error::{Error, Result};
use tracing::{debug, info};

use crate::{resolver, visitor::Acceptor, Expr, MutInterpreter, Stmt, Token, Value, Visitor};

pub type MutResolver = Rc<RefCell<Resolver>>;

pub struct Resolver {
    interpreter: MutInterpreter,
    pub scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    had_error: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FunctionType {
    None,
    Function,
}

impl Resolver {
    pub fn new(interpreter: &MutInterpreter) -> Resolver {
        Resolver {
            interpreter: interpreter.clone(),
            scopes: vec![],
            current_function: FunctionType::None,
            had_error: false,
        }
    }

    pub fn had_error(&self) -> bool {
        self.had_error
    }

    pub fn current_function(&self) -> FunctionType {
        self.current_function.clone()
    }

    pub fn replace_function(&mut self, replace: FunctionType) -> FunctionType {
        std::mem::replace(&mut self.current_function, replace)
    }

    pub fn resolve(self, stmts: &[Stmt]) -> Result<bool> {
        info!("Resolving statements");

        let resolver = Rc::new(RefCell::new(self));

        Self::resolve_block(&resolver.clone(), stmts)?;

        let had_error = resolver.borrow().had_error();

        Ok(had_error)
    }

    pub fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn resolve_block(visitor: &MutResolver, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            match stmt.accept(visitor) {
                Ok(_) => {}
                Err(e) => {
                    visitor.borrow_mut().had_error = true;
                    Self::error(&e)
                }
            };
        }

        Ok(())
    }

    fn error(e: &Error) {
        match e {
            Error::LocalVarReadWhileInitialized(token) => crate::report(
                token.line,
                "Can't read local variable in its own initializer",
            ),
            Error::RedefiningLocalVar(token) => crate::report(
                token.line,
                "Already a variable with this name in this scope",
            ),
            Error::TopLevelReturn(token) => {
                crate::report(token.line, "Can't return from top-level code")
            }
        }
    }

    pub fn declare(&mut self, name: &Token) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                return Err(Error::RedefiningLocalVar(name.clone()));
            }

            scope.insert(name.lexeme.clone(), false);
        }

        Ok(())
    }

    pub fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }

    pub fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter
                    .borrow_mut()
                    .resolve(expr, self.scopes.len() - 1 - i);
                return;
            }
        }
    }
}

impl Visitor<Result<()>> for &MutResolver {
    fn visit(&self, acceptor: impl Acceptor<Result<()>, Self>) -> Result<()>
    where
        Self: Sized,
    {
        acceptor.accept(self)
    }
}
