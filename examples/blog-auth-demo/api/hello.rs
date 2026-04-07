#[vega::get]
pub async fn handler() -> vega::server::ApiResponse {
    vega::server::ApiResponse::json(serde_json::json!({"message": "hello"}))
}

#[vega::post]
pub async fn create() -> vega::server::ApiResponse {
    vega::server::ApiResponse::json(serde_json::json!({"created": true}))
}
