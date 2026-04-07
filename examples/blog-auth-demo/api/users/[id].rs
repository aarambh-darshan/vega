#[vega::get]
pub async fn handler() -> vega::server::ApiResponse {
    vega::server::ApiResponse::json(serde_json::json!({"id": 1}))
}

#[vega::put]
pub async fn replace() -> vega::server::ApiResponse {
    vega::server::ApiResponse::json(serde_json::json!({"updated": true}))
}

#[vega::delete]
pub async fn delete() -> vega::server::ApiResponse {
    vega::server::ApiResponse::json(serde_json::json!({"deleted": true}))
}
