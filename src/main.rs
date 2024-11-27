pub type Result<T> = core::result::Result<T, Error>;

use std::env;
use std::fs;
use std::io::{self, Write};

use interpreter::Error;
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
            let mut scanner = Scanner::new(filename)?;

            scanner.scan_tokens()?;

            let tokens = scanner.tokens();

            for token in tokens {
                println!("{}", token);
            }

            Ok(())
        }
        _ => Err(Error::UnknownCommand(args[0].to_string()))?,
    }
}
