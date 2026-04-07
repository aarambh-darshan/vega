use crate::app::{current_user_from_headers, AppState};
use serde_json::json;
use vega::server::{
    web::{Body, Html, IntoResponse, Next, Redirect, Request, Response, State, StatusCode},
    ApiResponse,
};

pub(crate) async fn require_auth_page(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    match current_user_from_headers(req.headers(), &state).await {
        Some(user) => {
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        None => Redirect::to("/login").into_response(),
    }
}

pub(crate) async fn require_admin_page(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    match current_user_from_headers(req.headers(), &state).await {
        Some(user) if user.role == "admin" => {
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        Some(_) => (
            StatusCode::FORBIDDEN,
            Html(crate::pages::_layout::render_layout(
                "Forbidden",
                "<div class='error-page'><h1>403 — Forbidden</h1><p>You need admin privileges to access this page.</p><a href='/dashboard' class='btn'>Back to Dashboard</a></div>",
                None,
                None,
            )),
        )
            .into_response(),
        None => Redirect::to("/login").into_response(),
    }
}

pub(crate) async fn require_auth_api(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    match current_user_from_headers(req.headers(), &state).await {
        Some(user) => {
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        None => ApiResponse::status_json(
            StatusCode::UNAUTHORIZED,
            json!({"error": "authentication required"}),
        )
        .into_response(),
    }
}

pub(crate) async fn require_admin_api(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    match current_user_from_headers(req.headers(), &state).await {
        Some(user) if user.role == "admin" => {
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        Some(_) => ApiResponse::status_json(
            StatusCode::FORBIDDEN,
            json!({"error": "admin role required"}),
        )
        .into_response(),
        None => ApiResponse::status_json(
            StatusCode::UNAUTHORIZED,
            json!({"error": "authentication required"}),
        )
        .into_response(),
    }
}
