use crate::app::AppState;
use vega::server::{
    clear_session_cookie, parse_cookie_map,
    web::{IntoResponse, Redirect, Request, Response, State},
    SessionStore,
};

pub async fn handler(State(state): State<AppState>, req: Request) -> Response {
    let cookies = parse_cookie_map(req.headers());
    if let Some(token) = cookies.get("vega_session") {
        let _ = state.sessions.delete_session(token).await;
    }
    let cookie = clear_session_cookie(state.secure_cookie);
    let mut resp = Redirect::to("/").into_response();
    resp.headers_mut().insert(
        axum::http::header::SET_COOKIE,
        cookie.parse().expect("cookie header"),
    );
    resp
}
