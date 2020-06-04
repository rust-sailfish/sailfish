pub mod runtime;

pub use runtime::{RenderError, RenderResult};

pub trait TemplateOnce {
    fn render_once(self) -> runtime::RenderResult;
}

/// WIP
pub trait Template {
    fn render(self) -> runtime::RenderResult;
}
