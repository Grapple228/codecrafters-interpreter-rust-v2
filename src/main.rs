pub type Result<T> = core::result::Result<T, Error>;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

use interpreter::AstPrinter;
use interpreter::Error;
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
        _ => Err(Error::UnknownCommand(args[0].to_string()))?,
    }

    Ok(())
}

fn tokenize(filename: &str) -> Result<()> {
    let mut scanner = Scanner::new(filename)?;

    scanner.scan_tokens()?;

    for error in scanner.errors() {
        eprintln!("{}", error);
    }

    for token in scanner.tokens() {
        println!("{}", token);
    }

    if scanner.has_error() {
        process::exit(65)
    }

    Ok(())
}

fn parse(filename: &str) -> Result<()> {
    let mut scanner = Scanner::new(filename)?;

    scanner.scan_tokens()?;

    for error in scanner.errors() {
        eprintln!("{}", error);
    }

    if scanner.has_error() {
        process::exit(65)
    }

    let mut parser = Parser::new(&scanner.tokens());
    let expr = parser.parse();

    let printer = AstPrinter::default();
    let result = printer.print(expr);

    println!("{}", result);

    Ok(())
}
