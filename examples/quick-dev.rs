#![allow(unused)] // For beginning only.

use std::{io::stderr, process};

use anyhow::Result;
use interpreter::Scanner;
use tracing::info;

fn main() -> Result<()> {
    interpreter::init();

    let mut scanner = Scanner::new("test.lox")?;

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
