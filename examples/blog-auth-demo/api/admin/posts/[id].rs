use crate::app::{slugify, AppState, IdParam};
use serde::Deserialize;
use vega::server::{
    web::{Path, Request, State},
    ApiError, ApiRequest, ApiResponse,
};

#[derive(Debug, Deserialize)]
struct ReplacePostPayload {
    title: String,
    excerpt: String,
    body: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PatchPostPayload {
    title: Option<String>,
    excerpt: Option<String>,
    body: Option<String>,
    tags: Option<Vec<String>>,
}

#[vega::put(middleware = [auth::require_admin])]
pub async fn replace(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    req: Request,
) -> Result<ApiResponse, ApiError> {
    let mut api_req = ApiRequest::from_axum_request(req).await?;
    api_req.params.insert("id".to_string(), id.to_string());

    let params: IdParam =
        vega::use_params(&api_req.params).map_err(|error| ApiError::bad_request(error.to_string()))?;
    let payload: ReplacePostPayload = api_req.json()?;

    let mut posts = state.posts.lock().expect("posts lock");
    let Some(existing) = posts.iter_mut().find(|post| post.id == params.id) else {
        return Err(ApiError::bad_request("post not found"));
    };

    existing.title = payload.title;
    existing.excerpt = payload.excerpt;
    existing.body = payload.body;
    existing.tags = payload.tags;
    existing.slug = slugify(&existing.title);

    Ok(ApiResponse::json(serde_json::json!({"post": existing})))
}

#[vega::patch(middleware = [auth::require_admin])]
pub async fn patch(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    req: Request,
) -> Result<ApiResponse, ApiError> {
    let mut api_req = ApiRequest::from_axum_request(req).await?;
    api_req.params.insert("id".to_string(), id.to_string());

    let params: IdParam =
        vega::use_params(&api_req.params).map_err(|error| ApiError::bad_request(error.to_string()))?;
    let payload: PatchPostPayload = api_req.json()?;

    let mut posts = state.posts.lock().expect("posts lock");
    let Some(existing) = posts.iter_mut().find(|post| post.id == params.id) else {
        return Err(ApiError::bad_request("post not found"));
    };

    if let Some(title) = payload.title {
        existing.title = title;
        existing.slug = slugify(&existing.title);
    }
    if let Some(excerpt) = payload.excerpt {
        existing.excerpt = excerpt;
    }
    if let Some(body) = payload.body {
        existing.body = body;
    }
    if let Some(tags) = payload.tags {
        existing.tags = tags;
    }

    Ok(ApiResponse::json(serde_json::json!({"post": existing})))
}

#[vega::delete(middleware = [auth::require_admin])]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    req: Request,
) -> Result<ApiResponse, ApiError> {
    let mut api_req = ApiRequest::from_axum_request(req).await?;
    api_req.params.insert("id".to_string(), id.to_string());

    let params: IdParam =
        vega::use_params(&api_req.params).map_err(|error| ApiError::bad_request(error.to_string()))?;

    let mut posts = state.posts.lock().expect("posts lock");
    let before = posts.len();
    posts.retain(|post| post.id != params.id);

    if posts.len() == before {
        return Err(ApiError::bad_request("post not found"));
    }

    Ok(ApiResponse::json(serde_json::json!({"deleted": true, "id": params.id})))
}

mod auth {
    pub fn require_admin() {}
}
