#[vega::get]
pub async fn handler() -> vega::server::ApiResponse {
    vega::server::ApiResponse::json(serde_json::json!({"users": [{"id": 1, "name": "Aarambh"}]}))
}

#[vega::post]
pub async fn create() -> vega::server::ApiResponse {
    vega::server::ApiResponse::status_json(
        vega::server::web::StatusCode::CREATED,
        serde_json::json!({"id": 2}),
    )
}
