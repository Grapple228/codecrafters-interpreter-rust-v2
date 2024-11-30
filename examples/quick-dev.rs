#![allow(unused)] // For beginning only.

use std::{io::stderr, process};

use anyhow::Result;
use interpreter::{AstPrinter, Expr, Interpreter, Parser, Scanner, Token, TokenType, Value};
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
            .map(|s| printer.print(s.clone()))
            .collect::<Vec<_>>()
    );

    let mut interpreter = Interpreter::default();
    let result = interpreter.interpret_stmt(&stmts);

    if interpreter.had_runtime_error() {
        process::exit(70)
    }

    Ok(())
}
