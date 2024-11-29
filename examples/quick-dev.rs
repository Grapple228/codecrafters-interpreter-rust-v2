#![allow(unused)] // For beginning only.

use std::{io::stderr, process};

use anyhow::Result;
use interpreter::{AstPrinter, Expr, Parser, Scanner, Token, TokenType, Value};
use tracing::debug;

fn main() -> Result<()> {
    interpreter::init();

    let mut scanner = Scanner::new("test.lox")?;

    scanner.scan_tokens()?;

    for token in scanner.tokens() {
        println!("{}", token);
    }

    if scanner.had_error() {
        process::exit(65)
    }

    let mut parser = Parser::new(&scanner.tokens());
    let expr = parser.parse();

    match expr {
        Ok(expr) => {
            let printer = AstPrinter::default();
            let result = printer.print(expr);

            println!("{}", result);
        }
        Err(e) => process::exit(65),
    }

    Ok(())
}
