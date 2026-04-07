use crate::app::{current_user_from_headers, filter_posts, AppState, BlogQuery};
use vega::server::web::{Html, Request, State};

#[vega::page(mode = "ssr")]
pub fn BlogIndex() -> &'static str {
    "Blog"
}

pub fn render(state: &AppState, query: &BlogQuery) -> String {
    let posts = filter_posts(state, query);
    let per_page = query.per_page.unwrap_or(6);
    let current_page = query.page.unwrap_or(1).max(1);
    let total = posts.len();
    let total_pages = (total + per_page - 1) / per_page;
    let start = (current_page - 1) * per_page;
    let page_posts = posts.into_iter().skip(start).take(per_page);

    let mut cards = String::new();
    for post in page_posts {
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

        cards.push_str(&format!(
            r#"<div class="card">
                <h3><a href="/blog/{slug}">{title}</a></h3>
                <p>{excerpt}</p>
                <div style="display:flex;justify-content:space-between;align-items:center">
                    <div>{tags}</div>
                    <span style="font-size:0.75rem;color:var(--ink-muted)">by {author}</span>
                </div>
            </div>"#,
            slug = vega::core::html_escape(&post.slug),
            title = vega::core::html_escape(&post.title),
            excerpt = vega::core::html_escape(&post.excerpt),
            tags = tags,
            author = vega::core::html_escape(&post.author_email),
        ));
    }

    if cards.is_empty() {
        cards = r#"<div class="card"><p>No posts found matching your search.</p></div>"#.to_string();
    }

    // Pagination
    let mut pagination = String::new();
    if total_pages > 1 {
        pagination.push_str(r#"<div class="pagination">"#);
        for page in 1..=total_pages {
            let mut href = format!("/blog?page={page}");
            if let Some(q) = &query.q {
                href.push_str(&format!("&q={}", vega::core::html_escape(q)));
            }
            if let Some(tag) = &query.tag {
                href.push_str(&format!("&tag={}", vega::core::html_escape(tag)));
            }
            if page == current_page {
                pagination.push_str(&format!(r#"<span class="current">{page}</span>"#));
            } else {
                pagination.push_str(&format!(r#"<a href="{href}">{page}</a>"#));
            }
        }
        pagination.push_str("</div>");
    }

    let search_value = query.q.as_deref().unwrap_or("");
    let active_tag = query.tag.as_deref().unwrap_or("");

    format!(
        r#"<div style="display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:1rem;margin-bottom:1.5rem">
            <h1>Blog</h1>
            <form method="get" action="/blog" style="display:flex;gap:0.5rem">
                <input type="text" name="q" placeholder="Search posts..." value="{search}" style="min-width:200px" />
                <button type="submit" class="btn btn-outline btn-sm">Search</button>
            </form>
        </div>
        {active_tag_display}
        <div class="grid grid-2">{cards}</div>
        {pagination}
        <p style="margin-top:1rem;font-size:0.8rem;color:var(--ink-muted)">{total} post(s) total · Page {current} of {pages}</p>"#,
        search = vega::core::html_escape(search_value),
        active_tag_display = if active_tag.is_empty() {
            String::new()
        } else {
            format!(
                r#"<p style="margin-bottom:1rem">Filtered by tag: <span class="tag">{tag}</span> <a href="/blog">Clear</a></p>"#,
                tag = vega::core::html_escape(active_tag)
            )
        },
        cards = cards,
        pagination = pagination,
        total = total,
        current = current_page,
        pages = total_pages,
    )
}

pub async fn handler(State(state): State<AppState>, req: Request) -> Html<String> {
    let user = current_user_from_headers(req.headers(), &state).await;
    let query: BlogQuery = req
        .uri()
        .query()
        .and_then(|q| serde_urlencoded::from_str(q).ok())
        .unwrap_or_default();
    let body = render(&state, &query);
    Html(crate::pages::_layout::render_layout(
        "Blog",
        &body,
        user.as_ref(),
        None,
    ))
}
