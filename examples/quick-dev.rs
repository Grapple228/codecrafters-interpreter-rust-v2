#![allow(unused)] // For beginning only.

use anyhow::Result;
use interpreter::Scanner;
use tracing::info;

fn main() -> Result<()> {
    interpreter::init();

    let mut scanner = Scanner::new("test.lox")?;

    scanner.scan_tokens()?;

    let tokens = scanner.tokens();

    for token in tokens {
        println!("{}", token);
    }

    Ok(())
}
