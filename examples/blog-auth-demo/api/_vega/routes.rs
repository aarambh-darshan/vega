use crate::app::AppState;
use vega::server::{
    web::State,
    ApiResponse,
};

#[vega::get]
pub async fn handler(State(state): State<AppState>) -> ApiResponse {
    ApiResponse::json(state.manifest.clone())
}
