use crate::app::{unique_slug, AppState, Post};
use serde::Deserialize;
use vega::server::{
    web::{Request, State, StatusCode},
    ApiError, ApiRequest, ApiResponse, AuthUser,
};

#[derive(Debug, Deserialize)]
struct CreatePostPayload {
    title: String,
    excerpt: String,
    body: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[vega::get(middleware = [auth::require_admin])]
pub async fn handler(State(state): State<AppState>) -> Result<ApiResponse, ApiError> {
    let posts = state.posts.lock().expect("posts lock").clone();
    Ok(ApiResponse::json(serde_json::json!({"items": posts})))
}

#[vega::post(middleware = [auth::require_admin])]
pub async fn create(
    State(state): State<AppState>,
    req: Request,
) -> Result<ApiResponse, ApiError> {
    let user = req
        .extensions()
        .get::<AuthUser>()
        .cloned()
        .ok_or_else(|| ApiError::unauthorized("no active session"))?;

    let api_req = ApiRequest::from_axum_request(req).await?;
    let payload: CreatePostPayload = api_req.json()?;

    if payload.title.trim().is_empty() || payload.body.trim().is_empty() {
        return Err(ApiError::bad_request("title and body are required"));
    }

    let mut posts = state.posts.lock().expect("posts lock");
    let next_id = posts.iter().map(|post| post.id).max().unwrap_or(0) + 1;
    let slug = unique_slug(&posts, &payload.title, next_id);

    let post = Post {
        id: next_id,
        slug,
        title: payload.title,
        excerpt: payload.excerpt,
        body: payload.body,
        tags: payload.tags,
        author_email: user.email,
    };

    posts.push(post.clone());
    Ok(ApiResponse::status_json(
        StatusCode::CREATED,
        serde_json::json!({"post": post}),
    ))
}

mod auth {
    pub fn require_admin() {}
}
