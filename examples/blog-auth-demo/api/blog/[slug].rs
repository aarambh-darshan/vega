use crate::app::AppState;
use serde::Deserialize;
use vega::server::{
    web::{Path, Request, State},
    ApiError, ApiRequest, ApiResponse,
};

#[derive(Debug, Deserialize)]
struct BlogSlugParam {
    slug: String,
}

#[vega::get]
pub async fn handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    req: Request,
) -> Result<ApiResponse, ApiError> {
    let mut api_req = ApiRequest::from_axum_request(req).await?;
    api_req.params.insert("slug".to_string(), slug);

    let params: BlogSlugParam =
        vega::use_params(&api_req.params).map_err(|error| ApiError::bad_request(error.to_string()))?;

    let posts = state.posts.lock().expect("posts lock");
    let Some(post) = posts.iter().find(|post| post.slug == params.slug) else {
        return Err(ApiError::bad_request("post not found"));
    };

    Ok(ApiResponse::json(serde_json::json!({"post": post})))
}
