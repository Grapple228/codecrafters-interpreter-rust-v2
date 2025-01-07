use tracing::debug;
use tracing_subscriber::field::debug;

use crate::resolver::MutResolver;
use crate::{interpreter, resolver, value, MutInterpreter, TokenType, Value};
use crate::{visitor::Acceptor, AstPrinter, Token};

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
    Variable(Token),
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
}

impl Into<Stmt> for Expr {
    fn into(self) -> Stmt {
        Stmt::Expression(Box::new(self))
    }
}

impl Expr {
    pub fn name(&self) -> Option<String> {
        match self {
            Expr::Variable(token) => Some(token.lexeme.clone()),
            Expr::Assign { name, .. } => Some(name.lexeme.clone()),
            Expr::Binary { left, .. } => left.name(),
            Expr::Call { callee, .. } => callee.name(),
            _ => None,
        }
    }
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

impl Acceptor<resolver::Result<()>, &MutResolver> for Expr {
    fn accept(&self, visitor: &MutResolver) -> resolver::Result<()> {
        match self {
            Expr::Variable(token) => {
                if let Some(scope) = visitor.borrow().scopes.last() {
                    if let Some(value) = scope.get(&token.lexeme).cloned() {
                        if value == false {
                            return Err(resolver::Error::LocalVarReadWhileInitialized(
                                token.clone(),
                            ));
                        }
                    }
                }

                visitor.borrow_mut().resolve_local(self, token);

                Ok(())
            }
            Expr::Assign { name, value } => {
                value.accept(visitor)?;
                visitor.borrow_mut().resolve_local(value, name);

                Ok(())
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                left.accept(visitor)?;
                right.accept(visitor)?;

                Ok(())
            }
            Expr::Grouping(expr) => {
                expr.accept(visitor)?;
                Ok(())
            }
            Expr::Literal(value) => Ok(()),
            Expr::Unary { operator, right } => {
                right.accept(visitor)?;

                Ok(())
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                left.accept(visitor)?;
                right.accept(visitor)?;

                Ok(())
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                callee.accept(visitor)?;

                for argument in arguments {
                    argument.accept(visitor)?;
                }

                Ok(())
            }
        }
    }
}

impl Acceptor<interpreter::Result<Value>, &MutInterpreter> for Expr {
    fn accept(&self, visitor: &MutInterpreter) -> interpreter::Result<Value> {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = left.accept(visitor)?;
                let right = right.accept(visitor)?;

                Ok(left.calculate(Some(&right), operator)?)
            }
            Expr::Grouping(expr) => expr.accept(visitor),
            Expr::Literal(value) => {
                if let Some(value) = value {
                    Ok(value.to_owned())
                } else {
                    Ok(Value::Nil)
                }
            }
            Expr::Unary { operator, right } => {
                let value = right.accept(visitor)?;

                Ok(value.calculate(None, operator)?)
            }
            Expr::Variable(name) => {
                let interpreter = visitor.borrow();

                Ok(interpreter.look_up_variable(name)?)
            }
            Expr::Assign { name, value } => {
                let value = value.accept(visitor)?;

                let interpreter = visitor.borrow();

                if let Some(distance) = interpreter.locals.get(&name.lexeme).copied() {
                    interpreter.environment.borrow_mut().assign_at(
                        distance,
                        name,
                        Some(value.clone()),
                    );
                } else {
                    interpreter
                        .globals
                        .borrow_mut()
                        .assign(&name, Some(value.clone()));
                }

                Ok(value)
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = left.accept(visitor)?;

                if operator.token_type == TokenType::OR {
                    if left.is_truthy() {
                        return Ok(left);
                    }
                } else {
                    if !left.is_truthy() {
                        return Ok(left);
                    }
                }

                right.accept(visitor)
            }
            Expr::Call {
                callee,
                arguments,
                paren,
            } => {
                let callee = callee.accept(visitor)?;

                let arguments = arguments
                    .iter()
                    .map(|arg| arg.accept(visitor))
                    .collect::<interpreter::Result<Vec<Value>>>()?;

                if !callee.is_callable() {
                    return Err(value::Error::NotCallable {
                        token: paren.clone(),
                    })?;
                }

                let arity = callee.arity();
                if arguments.len() != arity {
                    return Err(value::Error::InvalidCountOfArguments {
                        token: paren.clone(),
                        count: arguments.len(),
                        expected: arity,
                    })?;
                }

                Ok(callee.call(paren, visitor, &arguments)?)
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
            } => Self::parenthesize(&visitor, &operator.lexeme, &[left, right]),
            Expr::Grouping(expr) => Self::parenthesize(&visitor, "group", &[expr]),
            Expr::Literal(value) => match value {
                None => panic!("Must not be None"),
                Some(Value::String(s)) => s.clone(),
                Some(Value::Number(n)) => format!("{:?}", n),
                Some(Value::Boolean(b)) => b.to_string(),
                Some(Value::Nil) => String::from("nil"),
                Some(Value::Callable(c)) => c.stringify(),
            },
            Expr::Unary { operator, right } => {
                Self::parenthesize(&visitor, &operator.lexeme, &[right])
            }
            Expr::Variable(name) => format!("{}", name.lexeme),
            Expr::Assign { name, value } => {
                format!("{} = {}", name.lexeme, value.accept(visitor))
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => Self::parenthesize(&visitor, &operator.lexeme, &[left, right]),
            Expr::Call {
                callee, arguments, ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|arg| arg.accept(visitor))
                    .collect::<Vec<String>>()
                    .join(", ");

                format!("{}({})", callee.accept(visitor), arguments)
            }
        }
    }
}
