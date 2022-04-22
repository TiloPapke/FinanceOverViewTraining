use askama::Template;
use axum::{response::{Html, Response, IntoResponse}, http::StatusCode};

#[derive(Template)]
#[template(path = "WelcomePage.html")]
pub struct MainPageTemplate {
   pub web_running_port: u16,
   pub additional_info:String,
}
pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}