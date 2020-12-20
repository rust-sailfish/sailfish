//! Sailfish runtime

#[macro_use]
mod utils;

mod buffer;
pub mod escape;
pub mod filter;
mod macros;
mod render;
mod size_hint;

pub use buffer::Buffer;
pub use render::{Render, RenderError, RenderResult};
pub use size_hint::SizeHint;

#[doc(hidden)]
pub use crate::{render, render_escaped, render_noop, render_text};
