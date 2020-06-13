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
