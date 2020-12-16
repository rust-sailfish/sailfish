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
#![cfg_attr(sailfish_nightly, feature(core_intrinsics))]

pub mod runtime;

pub use runtime::{RenderError, RenderResult};
#[cfg(feature = "derive")]
pub use sailfish_macros::TemplateOnce;

/// Template that can be rendered with consuming itself.
pub trait TemplateOnce: Sized {
    /// Render the template and return the rendering result as `RenderResult`
    ///
    /// This method never returns `Err`, unless you explicitly return RenderError
    /// inside templates
    #[inline]
    #[allow(deprecated)]
    fn render_once(self) -> runtime::RenderResult {
        let mut buf = String::new();
        self.render_once_to_string(&mut buf)?;
        Ok(buf)
    }

    /// Render the template and append the result to `buf`.
    ///
    /// This method never returns `Err`, unless you explicitly return RenderError
    /// inside templates
    #[deprecated(
        since = "0.2.1",
        note = "This function may be removed in the future due to performance issue"
    )]
    fn render_once_to_string(self, buf: &mut String) -> Result<(), RenderError>;
}

/// Work in Progress
pub trait Template {
    fn render(&self) -> runtime::RenderResult;
}
