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
