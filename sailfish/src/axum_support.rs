use super::TemplateOnce;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

/// Wrapper struct for TemplateOnce that allows us to have custom error messages and a better api for using the Axum framework
pub struct AxumTemplate<T: TemplateOnce> {
    template: T,
    custom_error_message: Option<String>,
}

impl<T: TemplateOnce> AxumTemplate<T> {
    /// Create an AxumTemplate from a TemplateOnce struct
    pub fn from(template: T) -> Self {
        Self {
            template,
            custom_error_message: None,
        }
    }

    /// Specify a custom error message to be rendered when template.render_once() returns an RenderError
    pub fn with_custom_error_message(mut self, message: String) -> Self {
        self.custom_error_message = Some(message);
        self
    }
}

impl<T> IntoResponse for AxumTemplate<T>
where
    T: TemplateOnce,
{
    fn into_response(self) -> Response {
        match self.template.render_once() {
            Ok(html) => Html(html).into_response(),
            Err(err) => {
                let error_message = self.custom_error_message.unwrap_or_else(|| {
                    format!("Failed to render template. Error: {err}")
                });
                (StatusCode::INTERNAL_SERVER_ERROR, error_message).into_response()
            }
        }
    }
}
