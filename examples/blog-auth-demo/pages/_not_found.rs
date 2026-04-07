use crate::app::{current_user_from_headers, AppState};
use vega::server::web::{Html, Request, State, StatusCode};

pub fn NotFound() -> &'static str {
    "404"
}

pub fn render(path: &str) -> String {
    format!(
        r#"<div class="error-page">
            <h1>404</h1>
            <p>No route matched <code>{}</code></p>
            <a href="/" class="btn btn-primary">Go Home</a>
        </div>"#,
        vega::core::html_escape(path)
    )
}

pub async fn handler(State(state): State<AppState>, req: Request) -> (StatusCode, Html<String>) {
    let user = current_user_from_headers(req.headers(), &state).await;
    let body = render(req.uri().path());
    (
        StatusCode::NOT_FOUND,
        Html(crate::pages::_layout::render_layout(
            "Not Found",
            &body,
            user.as_ref(),
            None,
        )),
    )
}
