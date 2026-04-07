use crate::app::{unique_slug, AppState, Post};
use vega::server::{
    web::{Html, IntoResponse, Request, Response, State},
    AuthUser,
};

#[vega::page(mode = "ssr", middleware = [auth::require_admin])]
pub fn AdminPage() -> &'static str {
    "Admin Panel"
}

#[derive(serde::Deserialize)]
pub struct CreatePostForm {
    title: String,
    excerpt: String,
    body: String,
    tags: String,
}

#[derive(serde::Deserialize)]
pub struct DeletePostForm {
    id: u64,
}

pub fn render(_user: &AuthUser, posts: &[Post], message: Option<&str>) -> String {
    let flash = match message {
        Some(msg) => format!(r#"<div class="flash">{}</div>"#, vega::core::html_escape(msg)),
        None => String::new(),
    };

    let mut rows = String::new();
    for post in posts {
        rows.push_str(&format!(
            r#"<tr>
                <td>{id}</td>
                <td><a href="/blog/{slug}">{title}</a></td>
                <td>{author}</td>
                <td>
                    <form method="post" action="/admin?action=delete" style="display:inline">
                        <input type="hidden" name="id" value="{id}" />
                        <button type="submit" class="btn btn-danger btn-sm">Delete</button>
                    </form>
                </td>
            </tr>"#,
            id = post.id,
            slug = vega::core::html_escape(&post.slug),
            title = vega::core::html_escape(&post.title),
            author = vega::core::html_escape(&post.author_email),
        ));
    }

    format!(
        r#"<h1>Admin Panel</h1>
        <p>Manage blog posts. Only admin users can access this page.</p>
        {flash}

        <h2>Create New Post</h2>
        <div class="card">
            <form method="post" action="/admin?action=create">
                <div class="form-group">
                    <label for="title">Title</label>
                    <input type="text" id="title" name="title" required />
                </div>
                <div class="form-group">
                    <label for="excerpt">Excerpt</label>
                    <input type="text" id="excerpt" name="excerpt" required />
                </div>
                <div class="form-group">
                    <label for="body">Body</label>
                    <textarea id="body" name="body" required></textarea>
                </div>
                <div class="form-group">
                    <label for="tags">Tags (comma-separated)</label>
                    <input type="text" id="tags" name="tags" placeholder="rust, web, vega" />
                </div>
                <button type="submit" class="btn btn-primary">Create Post</button>
            </form>
        </div>

        <h2>All Posts ({count})</h2>
        <div class="card" style="padding:0;overflow:hidden">
            <table>
                <thead><tr><th>ID</th><th>Title</th><th>Author</th><th>Actions</th></tr></thead>
                <tbody>{rows}</tbody>
            </table>
        </div>"#,
        flash = flash,
        rows = rows,
        count = posts.len(),
    )
}

pub async fn handler(State(state): State<AppState>, req: Request) -> Html<String> {
    let user = req.extensions().get::<AuthUser>().cloned();
    let posts = state.posts.lock().expect("posts lock").clone();

    let body = match &user {
        Some(u) => render(u, &posts, None),
        None => "<p>Not authenticated</p>".to_string(),
    };

    Html(crate::pages::_layout::render_layout(
        "Admin",
        &body,
        user.as_ref(),
        None,
    ))
}

pub async fn post_handler(State(state): State<AppState>, req: Request) -> Response {
    let action = req
        .uri()
        .query()
        .and_then(|q| {
            url::form_urlencoded::parse(q.as_bytes())
                .find(|(k, _)| k == "action")
                .map(|(_, v)| v.to_string())
        })
        .unwrap_or_default();

    // Extract user from extensions before consuming the request body
    let (parts, body) = req.into_parts();
    let user = parts.extensions.get::<AuthUser>().cloned();

    let user = match user {
        Some(u) => u,
        None => {
            return Html(crate::pages::_layout::render_layout(
                "Admin",
                "<p>Not authenticated</p>",
                None,
                None,
            ))
            .into_response();
        }
    };

    // Read body bytes
    let body_bytes = axum::body::to_bytes(body, 1024 * 1024)
        .await
        .unwrap_or_default();

    match action.as_str() {
        "create" => {
            let form: CreatePostForm = match serde_urlencoded::from_bytes(&body_bytes) {
                Ok(f) => f,
                Err(err) => {
                    let posts = state.posts.lock().expect("posts lock").clone();
                    let body = render(&user, &posts, Some(&format!("Invalid form data: {err}")));
                    return Html(crate::pages::_layout::render_layout(
                        "Admin",
                        &body,
                        Some(&user),
                        None,
                    ))
                    .into_response();
                }
            };

            let tags: Vec<String> = form
                .tags
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            let mut posts = state.posts.lock().expect("posts lock");
            let next_id = posts.iter().map(|p| p.id).max().unwrap_or(0) + 1;
            let slug = unique_slug(&posts, &form.title, next_id);
            posts.push(Post {
                id: next_id,
                slug,
                title: form.title,
                excerpt: form.excerpt,
                body: form.body,
                tags,
                author_email: user.email.clone(),
            });
            let all_posts = posts.clone();
            drop(posts);

            let body = render(&user, &all_posts, Some("Post created successfully!"));
            Html(crate::pages::_layout::render_layout(
                "Admin",
                &body,
                Some(&user),
                None,
            ))
            .into_response()
        }
        "delete" => {
            let form: DeletePostForm = match serde_urlencoded::from_bytes(&body_bytes) {
                Ok(f) => f,
                Err(_) => {
                    let posts = state.posts.lock().expect("posts lock").clone();
                    let body = render(&user, &posts, Some("Invalid delete request"));
                    return Html(crate::pages::_layout::render_layout(
                        "Admin",
                        &body,
                        Some(&user),
                        None,
                    ))
                    .into_response();
                }
            };

            let mut posts = state.posts.lock().expect("posts lock");
            let before = posts.len();
            posts.retain(|p| p.id != form.id);
            let deleted = posts.len() < before;
            let all_posts = posts.clone();
            drop(posts);

            let msg = if deleted {
                format!("Post #{} deleted.", form.id)
            } else {
                format!("Post #{} not found.", form.id)
            };

            let body = render(&user, &all_posts, Some(&msg));
            Html(crate::pages::_layout::render_layout(
                "Admin",
                &body,
                Some(&user),
                None,
            ))
            .into_response()
        }
        _ => {
            let posts = state.posts.lock().expect("posts lock").clone();
            let body = render(&user, &posts, None);
            Html(crate::pages::_layout::render_layout(
                "Admin",
                &body,
                Some(&user),
                None,
            ))
            .into_response()
        }
    }
}

mod auth {
    pub fn require_admin() {}
}
