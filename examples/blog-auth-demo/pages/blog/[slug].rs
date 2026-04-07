use crate::app::{current_user_from_headers, AppState, Post};
use vega::server::web::{Html, Path, Request, State, StatusCode};

#[vega::page(mode = "ssr")]
pub fn BlogPost() -> &'static str {
    "Blog Post"
}

pub fn render(post: &Post) -> String {
    let tags: String = post
        .tags
        .iter()
        .map(|tag| {
            format!(
                r#"<a href="/blog?tag={tag}" class="tag">{tag}</a>"#,
                tag = vega::core::html_escape(tag)
            )
        })
        .collect();

    format!(
        r#"<article>
            <div style="margin-bottom:0.75rem">
                <a href="/blog" style="font-size:0.875rem;color:var(--ink-muted)">← Back to Blog</a>
            </div>
            <h1>{title}</h1>
            <div style="display:flex;gap:0.75rem;align-items:center;margin-bottom:1.5rem;color:var(--ink-muted);font-size:0.875rem">
                <span>By {author}</span>
                <span>·</span>
                <div>{tags}</div>
            </div>
            <div style="font-size:1rem;line-height:1.8;color:var(--ink)">
                <p>{body}</p>
            </div>
        </article>"#,
        title = vega::core::html_escape(&post.title),
        author = vega::core::html_escape(&post.author_email),
        tags = tags,
        body = vega::core::html_escape(&post.body),
    )
}

pub async fn handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    req: Request,
) -> (StatusCode, Html<String>) {
    let user = current_user_from_headers(req.headers(), &state).await;

    let post = {
        let posts = state.posts.lock().expect("posts lock");
        posts.iter().find(|p| p.slug == slug).cloned()
    };

    let Some(post) = post else {
        let body = format!(
            r#"<div class="error-page">
                <h1>404</h1>
                <p>Post <code>{}</code> not found.</p>
                <a href="/blog" class="btn btn-primary">Back to Blog</a>
            </div>"#,
            vega::core::html_escape(&slug)
        );
        return (
            StatusCode::NOT_FOUND,
            Html(crate::pages::_layout::render_layout(
                "Not Found",
                &body,
                user.as_ref(),
                None,
            )),
        );
    };

    let body = render(&post);
    (
        StatusCode::OK,
        Html(crate::pages::_layout::render_layout(
            &post.title,
            &body,
            user.as_ref(),
            None,
        )),
    )
}

#[vega::server_fn(cache = 60)]
async fn get_post(slug: String) -> Result<String, vega::FetchError> {
    Ok(format!("post:{slug}"))
}
