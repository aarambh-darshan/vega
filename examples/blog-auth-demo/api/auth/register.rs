use crate::app::AppState;
use serde::Deserialize;
use vega::server::{
    web::{State, StatusCode},
    ApiError, ApiRequest, ApiResponse, AuthService,
};

#[derive(Debug, Deserialize)]
struct RegisterPayload {
    email: String,
    password: String,
}

#[vega::post]
pub async fn create(
    State(state): State<AppState>,
    req: vega::server::web::Request,
) -> Result<ApiResponse, ApiError> {
    let api_req = ApiRequest::from_axum_request(req).await?;
    let payload: RegisterPayload = api_req.json()?;

    if payload.email.trim().is_empty() || payload.password.trim().is_empty() {
        return Err(ApiError::bad_request("email and password are required"));
    }

    let user = state
        .auth
        .register(&payload.email, &payload.password, "member")
        .await
        .map_err(|error| ApiError::internal(format!("register failed: {error}")))?;

    Ok(ApiResponse::status_json(
        StatusCode::CREATED,
        serde_json::json!({"user": user}),
    ))
}
