use vega::server::{ApiError, ApiResponse, AuthUser};

#[vega::get]
pub async fn handler(req: vega::server::web::Request) -> Result<ApiResponse, ApiError> {
    let Some(user) = req.extensions().get::<AuthUser>().cloned() else {
        return Err(ApiError::unauthorized("no active session"));
    };

    Ok(ApiResponse::json(serde_json::json!({"user": user})))
}
