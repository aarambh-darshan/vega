use crate::app::{filter_posts, AppState, BlogQuery};
use vega::server::{
    web::{Request, State},
    ApiError, ApiRequest, ApiResponse,
};

#[vega::get]
pub async fn handler(State(state): State<AppState>, req: Request) -> Result<ApiResponse, ApiError> {
    let api_req = ApiRequest::from_axum_request(req).await?;
    let query = api_req.query::<BlogQuery>().unwrap_or_default();

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(10).clamp(1, 50);

    let filtered = filter_posts(&state, &query);
    let total = filtered.len();
    let total_pages = if total == 0 { 1 } else { total.div_ceil(per_page) };
    let page = page.min(total_pages);
    let start = (page - 1) * per_page;

    let items = filtered
        .into_iter()
        .skip(start)
        .take(per_page)
        .collect::<Vec<_>>();

    Ok(ApiResponse::json(serde_json::json!({
        "page": page,
        "per_page": per_page,
        "total": total,
        "total_pages": total_pages,
        "items": items,
    })))
}
