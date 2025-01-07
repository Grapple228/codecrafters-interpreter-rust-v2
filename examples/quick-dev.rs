#![allow(unused)] // For beginning only.

use std::{cell::RefCell, io::stderr, process, rc::Rc};

type Error = Box<dyn std::error::Error>;
type Result<T> = core::result::Result<T, Error>; // For tests.

use interpreter::{
    AstPrinter, Expr, Interpreter, Parser, Resolver, Scanner, Token, TokenType, Value,
};
use tracing::debug;

fn main() -> Result<()> {
    interpreter::init();

    let mut scanner = Scanner::new("test.lox")?;

    scanner.scan_tokens()?;

    debug!(
        "Scanned tokens: {:#?}",
        scanner
            .tokens()
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
    );

    if scanner.had_error() {
        process::exit(65)
    }

    let mut parser = Parser::new(&scanner.tokens());
    let stmts = parser.parse_stmt();

    if parser.had_error() {
        process::exit(65)
    }

    let stmts = stmts?;

    let printer = AstPrinter::default();
    debug!(
        "Parsed expressions: {:#?}",
        stmts
            .iter()
            .flat_map(|s| printer
                .print(s)
                .split('\n')
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );

    let mut interpreter = Rc::new(RefCell::from(Interpreter::default()));

    let mut resolver = Resolver::new(&interpreter);
    if resolver.resolve(&stmts)? {
        process::exit(65)
    }

    let result = interpreter.borrow_mut().interpret_stmt(&stmts);

    if interpreter.borrow().had_runtime_error() {
        process::exit(70)
    }

    Ok(())
}
