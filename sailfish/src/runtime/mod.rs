//! Sailfish runtime

mod buffer;
pub mod escape;
mod integer;
mod macros;
mod render;
mod size_hint;

pub use buffer::*;
pub use render::*;
pub use size_hint::*;

use std::fmt;

#[doc(hidden)]
pub use crate::{render, render_escaped, render_noop, render_text};

/// The error type which is returned from template function
#[derive(Clone, Debug)]
pub struct RenderError {
    // currently RenderError simply wraps the fmt::Error
    inner: fmt::Error,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

impl From<fmt::Error> for RenderError {
    #[inline]
    fn from(other: fmt::Error) -> Self {
        Self { inner: other }
    }
}

pub type RenderResult = Result<String, RenderError>;

pub struct Context {
    #[doc(hidden)]
    pub buf: Buffer,
}

impl Context {
    #[inline]
    pub fn into_result(self) -> RenderResult {
        Ok(self.buf.into_string())
    }
}
