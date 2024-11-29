#![allow(unused)] // For beginning only.

// region:    --- Modules

use std::{fs, path::Path, usize};

use tracing::{debug, info};
use tracing_subscriber::{fmt::format, EnvFilter};

// -- Modules
mod config;
mod error;
mod extensions;
mod interpreter;
mod parser;
mod printer;
mod scanner;
mod token;
mod tree;
mod value;
mod visitor;

// -- Flatten
pub use config::config;
pub use error::{Error, Result};
pub use interpreter::Interpreter;
pub use parser::Parser;
pub use printer::AstPrinter;
pub use scanner::Scanner;
pub use token::{Token, TokenType};
pub use tree::{Expr, Stmt};
pub use value::Value;
pub use visitor::Visitor;

// endregion: --- Modules

pub fn report(line: usize, message: impl Into<String>) {
    eprintln!("[line {}] Error: {}", line, message.into());
}

pub fn init() -> Result<()> {
    // LOGGING INITIALIZATION
    tracing_subscriber::fmt()
        .without_time() // For early development
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Initializing");

    // CONFIG INITIALIZATION
    info!("Loading config...");
    let config = config();

    Ok(())
}
