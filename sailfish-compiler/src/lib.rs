#![forbid(unsafe_code)]

#[macro_use]
mod error;

mod analyzer;
mod compiler;
mod config;
mod optimizer;
mod parser;
mod resolver;
mod translator;
mod util;

pub use compiler::Compiler;
pub use config::Config;
pub use error::{Error, ErrorKind};

#[cfg(feature = "procmacro")]
#[doc(hidden)]
pub mod procmacro;
