//! Sailfish is a simple, small, and extremely fast template engine for Rust.
//! Before reading this reference,
//! I recommend reading [User guide](https://sailfish.netlify.app/en/).
//!
//! This crate contains utilities for rendering sailfish template.
//! If you want to use sailfish templates, import `sailfish-macros` crate and use
//! derive macro `#[derive(TemplateOnce)]` or `#[derive(Template)]`.
//!
//! In most cases you don't need to care about the `runtime` module in this crate, but
//! if you want to render custom data inside templates, you must implement
//! `runtime::Render` trait for that type.
//!
//! ```ignore
//! #[macro_use]
//! extern crate sailfish_macros;
//!
//! use sailfish::TemplateOnce;
//!
//! #[derive(TemplateOnce)]
//! #[template(path = "hello.stpl")]
//! struct HelloTemplate {
//!     messages: Vec<String>
//! }
//!
//! fn main() {
//!     let ctx = HelloTemplate {
//!         messages: vec!["foo".to_string(), "bar".to_string()]
//!     };
//!
//!     println!("{}", ctx.render_once().unwrap());
//! }
//! ```

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/Kogia-sima/sailfish/master/resources/icon.png"
)]

pub mod runtime;

pub use runtime::{RenderError, RenderResult};

/// Template that can be rendered with consuming itself.
pub trait TemplateOnce {
    fn render_once(self) -> runtime::RenderResult;
}

/// WIP
pub trait Template {
    fn render(&self) -> runtime::RenderResult;
}
