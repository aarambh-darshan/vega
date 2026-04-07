use crate::app::AppState;
use serde::Deserialize;
use vega::server::{
    make_session_cookie,
    web::State,
    ApiError, ApiRequest, ApiResponse, AuthService, SessionStore,
};

#[derive(Debug, Deserialize)]
struct LoginPayload {
    email: String,
    password: String,
}

#[vega::post]
pub async fn create(
    State(state): State<AppState>,
    req: vega::server::web::Request,
) -> Result<ApiResponse, ApiError> {
    let api_req = ApiRequest::from_axum_request(req).await?;
    let payload: LoginPayload = api_req.json()?;

    let Some(user) = state
        .auth
        .login(&payload.email, &payload.password)
        .await
        .map_err(|error| ApiError::internal(format!("login failed: {error}")))?
    else {
        return Err(ApiError::unauthorized("invalid credentials"));
    };

    let token = state
        .sessions
        .create_session(user.clone())
        .await
        .map_err(|error| ApiError::internal(format!("session failed: {error}")))?;

    let cookie = make_session_cookie(&token, state.secure_cookie, 60 * 60 * 24 * 7);
    ApiResponse::json(serde_json::json!({"user": user})).with_cookie(&cookie)
}
