#![allow(unused)] // For beginning only.

// region:    --- Modules

use std::{fs, path::Path};

use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

// -- Modules
mod config;
mod error;
mod scanner;
mod string_ext;
mod token;

// -- Flatten
pub use config::config;
pub use error::{Error, Result, SourceError};
pub use scanner::Scanner;
pub use string_ext::StringExt;
pub use token::{Token, TokenType};

// endregion: --- Modules

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
