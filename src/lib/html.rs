use askama::Template;
use axum::{
    debug_handler,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Form, Json,
};
use tracing::log::info;

#[derive(Template)]
#[template(path = "base.html")]
struct HomepageTemplate;

#[debug_handler]
pub async fn homepage() -> impl IntoResponse {
    HtmlTemplate(HomepageTemplate {})
}

pub struct HtmlTemplate<T>(T);
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),

            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed To Template HTML: {}", e),
            )
                .into_response(),
        }
    }
}

