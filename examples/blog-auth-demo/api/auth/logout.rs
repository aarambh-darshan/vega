use crate::app::AppState;
use vega::server::{
    clear_session_cookie,
    web::State,
    ApiError, ApiRequest, ApiResponse, SessionStore,
};

#[vega::post]
pub async fn create(
    State(state): State<AppState>,
    req: vega::server::web::Request,
) -> Result<ApiResponse, ApiError> {
    let api_req = ApiRequest::from_axum_request(req).await?;
    if let Some(token) = api_req.cookie("vega_session") {
        let _ = state.sessions.delete_session(token).await;
    }

    let cookie = clear_session_cookie(state.secure_cookie);
    ApiResponse::json(serde_json::json!({"ok": true})).with_cookie(&cookie)
}
