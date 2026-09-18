//! Sailfish is a simple, small, and extremely fast template engine for Rust.
//! Before reading this reference,
//! I recommend reading [User guide](https://rust-sailfish.github.io/sailfish/).
//!
//! This crate contains utilities for rendering Sailfish templates. With the default
//! `derive` feature, it also re-exports the `TemplateSimple`, `TemplateOnce`,
//! `TemplateMut`, and `Template` derive macros. A separate dependency on
//! `sailfish-macros` is not required.
//!
//! Use [`TemplateSimple`] for direct field access, or [`TemplateOnce`],
//! [`TemplateMut`], and [`Template`] to access fields and methods through `self`.
//! The latter traits render by consuming the context, mutably borrowing it, or
//! borrowing it through a shared reference, respectively.
//!
//! In most cases you don't need to care about the `runtime` module in this crate, but
//! if you want to render custom data inside templates, you must implement
//! `runtime::Render` trait for that type.
//!
//! ```
//! # #[cfg(feature = "derive")]
//! # fn main() -> Result<(), sailfish::RenderError> {
//! use sailfish::Template;
//!
//! #[derive(Template)]
//! #[template(source = "<% for msg in &self.messages { %><div><%= msg %></div><% } %>")]
//! struct HelloTemplate {
//!     messages: Vec<String>,
//! }
//!
//! let ctx = HelloTemplate {
//!     messages: vec!["foo".to_string(), "bar".to_string()],
//! };
//!
//! assert_eq!(ctx.render()?, "<div>foo</div><div>bar</div>");
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "derive"))]
//! # fn main() {}
//! ```

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/rust-sailfish/sailfish/main/resources/icon.png"
)]

#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::redundant_closure)]
#![deny(missing_docs)]

pub mod runtime;

use runtime::Buffer;
pub use runtime::{RenderError, RenderResult};
#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use sailfish_macros::{Template, TemplateMut, TemplateOnce, TemplateSimple};

/// Template that consumes its context and exposes fields without using `self`.
///
/// The derive macro makes fields available as local variables. Use [`TemplateOnce`]
/// instead if the template needs to access fields or methods through `self`.
pub trait TemplateSimple: Sized {
    /// Render the template and return the rendering result as `RenderResult`
    ///
    /// Returns an error if rendering a value, applying a filter, rendering a nested
    /// template, or executing template code fails.
    ///
    /// When you use `render_once` method, total rendered size will be cached, and at
    /// the next time, buffer will be pre-allocated based on the cached length.
    ///
    /// If you don't want this behaviour, you can use `render_once_to` method instead.
    fn render_once(self) -> runtime::RenderResult;

    /// Render the template and append the result to `buf`.
    ///
    /// Returns an error if rendering a value, applying a filter, rendering a nested
    /// template, or executing template code fails. On error, `buf` may contain
    /// partial output; changes to the buffer are not rolled back.
    ///
    /// ```
    /// use sailfish::TemplateSimple;
    /// use sailfish::runtime::Buffer;
    ///
    /// # pub struct HelloTemplate {
    /// #   messages: Vec<String>,
    /// # }
    /// #
    /// # impl TemplateSimple for HelloTemplate {
    /// #     fn render_once(self) -> Result<String, sailfish::RenderError> {
    /// #         Ok(String::new())
    /// #     }
    /// #
    /// #     fn render_once_to(self, buf: &mut Buffer)
    /// #             -> Result<(), sailfish::RenderError> {
    /// #         Ok(())
    /// #     }
    /// # }
    /// #
    /// let tpl = HelloTemplate {
    ///     messages: vec!["foo".to_string()]
    /// };
    ///
    /// // custom pre-allocation
    /// let mut buffer = Buffer::with_capacity(100);
    /// tpl.render_once_to(&mut buffer).unwrap();
    /// ```
    fn render_once_to(self, buf: &mut Buffer) -> Result<(), RenderError>;
}

/// Template that consumes its context and accesses fields and methods through `self`.
pub trait TemplateOnce: Sized {
    /// Render the template and return the rendering result as `RenderResult`
    ///
    /// Returns an error if rendering a value, applying a filter, rendering a nested
    /// template, or executing template code fails.
    ///
    /// When you use `render_once` method, total rendered size will be cached, and at
    /// the next time, buffer will be pre-allocated based on the cached length.
    ///
    /// If you don't want this behaviour, you can use `render_once_to` method instead.
    fn render_once(self) -> runtime::RenderResult;

    /// Render the template and append the result to `buf`.
    ///
    /// Returns an error if rendering a value, applying a filter, rendering a nested
    /// template, or executing template code fails. On error, `buf` may contain
    /// partial output; changes to the buffer are not rolled back.
    ///
    /// ```
    /// use sailfish::TemplateOnce;
    /// use sailfish::runtime::Buffer;
    ///
    /// # pub struct HelloTemplate {
    /// #   messages: Vec<String>,
    /// # }
    /// #
    /// # impl TemplateOnce for HelloTemplate {
    /// #     fn render_once(self) -> Result<String, sailfish::RenderError> {
    /// #         Ok(String::new())
    /// #     }
    /// #
    /// #     fn render_once_to(self, buf: &mut Buffer)
    /// #             -> Result<(), sailfish::RenderError> {
    /// #         Ok(())
    /// #     }
    /// # }
    /// #
    /// let tpl = HelloTemplate {
    ///     messages: vec!["foo".to_string()]
    /// };
    ///
    /// // custom pre-allocation
    /// let mut buffer = Buffer::with_capacity(100);
    /// tpl.render_once_to(&mut buffer).unwrap();
    /// ```
    fn render_once_to(self, buf: &mut Buffer) -> Result<(), RenderError>;
}

/// Template that can be rendered repeatedly through a mutable reference.
///
/// Templates access fields and methods through `self` and may mutate the context.
/// Deriving this trait also implements [`TemplateOnce`].
pub trait TemplateMut: TemplateOnce {
    /// Render the template and return the rendering result as `RenderResult`
    ///
    /// Returns an error if rendering a value, applying a filter, rendering a nested
    /// template, or executing template code fails.
    ///
    /// When you use `render_mut` method, total rendered size will be cached, and at
    /// the next time, buffer will be pre-allocated based on the cached length.
    ///
    /// If you don't want this behaviour, you can use `render_mut_to` method instead.
    fn render_mut(&mut self) -> runtime::RenderResult;

    /// Render the template and append the result to `buf`.
    ///
    /// Returns an error if rendering a value, applying a filter, rendering a nested
    /// template, or executing template code fails. On error, `buf` may contain
    /// partial output; changes to the buffer are not rolled back.
    ///
    /// ```
    /// use sailfish::{TemplateOnce, TemplateMut};
    /// use sailfish::runtime::Buffer;
    ///
    /// # pub struct HelloTemplate {
    /// #   messages: Vec<String>,
    /// # }
    /// #
    /// # impl TemplateOnce for HelloTemplate {
    /// #     fn render_once(self) -> Result<String, sailfish::RenderError> {
    /// #         Ok(String::new())
    /// #     }
    /// #
    /// #     fn render_once_to(self, buf: &mut Buffer)
    /// #             -> Result<(), sailfish::RenderError> {
    /// #         Ok(())
    /// #     }
    /// # }
    /// #
    /// # impl TemplateMut for HelloTemplate {
    /// #     fn render_mut(&mut self) -> Result<String, sailfish::RenderError> {
    /// #         Ok(String::new())
    /// #     }
    /// #
    /// #     fn render_mut_to(&mut self, buf: &mut Buffer)
    /// #             -> Result<(), sailfish::RenderError> {
    /// #         Ok(())
    /// #     }
    /// # }
    /// #
    /// let mut tpl = HelloTemplate {
    ///     messages: vec!["foo".to_string()]
    /// };
    ///
    /// // custom pre-allocation
    /// let mut buffer = Buffer::with_capacity(100);
    /// tpl.render_mut_to(&mut buffer).unwrap();
    /// ```
    fn render_mut_to(&mut self, buf: &mut Buffer) -> Result<(), RenderError>;
}

/// Template that can be rendered repeatedly through a shared reference.
///
/// Templates access fields and methods through `self`. Deriving this trait also
/// implements [`TemplateMut`] and [`TemplateOnce`].
pub trait Template: TemplateMut {
    /// Render the template and return the rendering result as `RenderResult`
    ///
    /// Returns an error if rendering a value, applying a filter, rendering a nested
    /// template, or executing template code fails.
    ///
    /// When you use `render` method, total rendered size will be cached, and at
    /// the next time, buffer will be pre-allocated based on the cached length.
    ///
    /// If you don't want this behaviour, you can use `render_to` method instead.
    fn render(&self) -> runtime::RenderResult;

    /// Render the template and append the result to `buf`.
    ///
    /// Returns an error if rendering a value, applying a filter, rendering a nested
    /// template, or executing template code fails. On error, `buf` may contain
    /// partial output; changes to the buffer are not rolled back.
    ///
    /// ```
    /// use sailfish::{TemplateOnce, TemplateMut, Template};
    /// use sailfish::runtime::Buffer;
    ///
    /// # pub struct HelloTemplate {
    /// #   messages: Vec<String>,
    /// # }
    /// #
    /// # impl TemplateOnce for HelloTemplate {
    /// #     fn render_once(self) -> Result<String, sailfish::RenderError> {
    /// #         Ok(String::new())
    /// #     }
    /// #
    /// #     fn render_once_to(self, buf: &mut Buffer)
    /// #             -> Result<(), sailfish::RenderError> {
    /// #         Ok(())
    /// #     }
    /// # }
    /// #
    /// # impl TemplateMut for HelloTemplate {
    /// #     fn render_mut(&mut self) -> Result<String, sailfish::RenderError> {
    /// #         Ok(String::new())
    /// #     }
    /// #
    /// #     fn render_mut_to(&mut self, buf: &mut Buffer)
    /// #             -> Result<(), sailfish::RenderError> {
    /// #         Ok(())
    /// #     }
    /// # }
    /// #
    /// # impl Template for HelloTemplate {
    /// #     fn render(&self) -> Result<String, sailfish::RenderError> {
    /// #         Ok(String::new())
    /// #     }
    /// #
    /// #     fn render_to(&self, buf: &mut Buffer)
    /// #             -> Result<(), sailfish::RenderError> {
    /// #         Ok(())
    /// #     }
    /// # }
    /// #
    /// let tpl = HelloTemplate {
    ///     messages: vec!["foo".to_string()]
    /// };
    ///
    /// // custom pre-allocation
    /// let mut buffer = Buffer::with_capacity(100);
    /// tpl.render_to(&mut buffer).unwrap();
    /// ```
    fn render_to(&self, buf: &mut Buffer) -> Result<(), RenderError>;
}
