// region:    --- Modules

use std::usize;

use tracing::info;
use tracing_subscriber::EnvFilter;

// -- Modules
mod config;
mod error;
mod extensions;
mod interpreter;
mod parser;
mod printer;
mod resolver;
mod scanner;
mod token;
mod tree;
mod value;
mod visitor;

// -- Flatten
pub use config::config;
pub use error::{Error, Result};
pub use interpreter::{Interpreter, MutInterpreter};
pub use parser::Parser;
pub use printer::AstPrinter;
pub use resolver::Resolver;
pub use scanner::Scanner;
pub use token::{Token, TokenType};
pub use tree::{Expr, Stmt};
pub use value::{Callable, CallableFn, Value};
pub use visitor::Visitor;

// endregion: --- Modules

pub struct W<T>(pub T);

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
    let _ = config();

    Ok(())
}
