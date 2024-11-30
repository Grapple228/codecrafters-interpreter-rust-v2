pub type Result<T> = core::result::Result<T, Error>;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

use interpreter::AstPrinter;
use interpreter::Error;
use interpreter::Interpreter;
use interpreter::Parser;
use interpreter::Scanner;

fn main() -> Result<()> {
    interpreter::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        Err(Error::ProgramExecutionError(format!(
            "Usage: {} tokenize <filename>",
            args[0]
        )))?;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            tokenize(filename)?;
        }
        "parse" => {
            parse(filename)?;
        }
        "evaluate" => {
            evaluate(filename)?;
        }
        "run" => {
            run(filename)?;
        }
        _ => Err(Error::UnknownCommand(args[0].to_string()))?,
    }

    Ok(())
}

fn tokenize(filename: &str) -> Result<()> {
    let mut scanner = Scanner::new(filename)?;

    scanner.scan_tokens()?;

    for token in scanner.tokens() {
        println!("{}", token);
    }

    if scanner.had_error() {
        process::exit(65)
    }

    Ok(())
}

fn parse(filename: &str) -> Result<()> {
    let mut scanner = Scanner::new(filename)?;

    scanner.scan_tokens()?;

    if scanner.had_error() {
        process::exit(65)
    }

    let mut parser = Parser::new(&scanner.tokens());
    let expr = parser.parse_expr();

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

fn evaluate(filename: &str) -> Result<()> {
    let mut scanner = Scanner::new(filename)?;

    scanner.scan_tokens()?;

    if scanner.had_error() {
        process::exit(65)
    }

    let mut parser = Parser::new(&scanner.tokens());
    let expr = parser.parse_expr();

    if parser.had_error() {
        process::exit(65)
    }

    let mut interpreter = Interpreter::default();
    let result = interpreter.interpret(expr?);

    if interpreter.had_runtime_error() {
        process::exit(70)
    }

    match result {
        Ok(value) => {
            println!("{}", value.stringify());
        }
        Err(e) => process::exit(70),
    }

    Ok(())
}

fn run(filename: &str) -> Result<()> {
    todo!()
}
