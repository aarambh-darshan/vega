use crate::app::{AppState, Post};
use vega::server::{
    web::{Html, Request, State},
    AuthUser,
};

#[vega::page(mode = "csr", middleware = [auth::require_auth])]
pub fn DashboardPage() -> &'static str {
    "CSR dashboard"
}

pub fn render(user: &AuthUser, posts: &[Post], total_posts: usize) -> String {
    let mut recent = String::new();
    for post in posts.iter().take(5) {
        recent.push_str(&format!(
            r#"<tr>
                <td><a href="/blog/{slug}">{title}</a></td>
                <td>{author}</td>
                <td>{tags}</td>
            </tr>"#,
            slug = vega::core::html_escape(&post.slug),
            title = vega::core::html_escape(&post.title),
            author = vega::core::html_escape(&post.author_email),
            tags = post
                .tags
                .iter()
                .map(|t| format!(r#"<span class="tag">{}</span>"#, vega::core::html_escape(t)))
                .collect::<String>(),
        ));
    }

    format!(
        r#"<h1>Dashboard</h1>
        <p>Welcome back, <strong>{email}</strong></p>

        <div class="grid grid-3" style="margin-top:1.5rem">
            <div class="card stat">
                <div class="stat-value">{total}</div>
                <div class="stat-label">Total Posts</div>
            </div>
            <div class="card stat">
                <div class="stat-value">{role}</div>
                <div class="stat-label">Your Role</div>
            </div>
            <div class="card stat">
                <div class="stat-value">✓</div>
                <div class="stat-label">Session Active</div>
            </div>
        </div>

        <h2>Recent Posts</h2>
        <div class="card" style="padding:0;overflow:hidden">
            <table>
                <thead><tr><th>Title</th><th>Author</th><th>Tags</th></tr></thead>
                <tbody>{recent}</tbody>
            </table>
        </div>

        <div style="margin-top:1.5rem;display:flex;gap:0.75rem">
            <a href="/blog" class="btn btn-outline">View Blog</a>
            {admin_link}
            <form method="post" action="/logout">
                <button type="submit" class="btn btn-outline">Sign Out</button>
            </form>
        </div>"#,
        email = vega::core::html_escape(&user.email),
        total = total_posts,
        role = vega::core::html_escape(&user.role),
        recent = recent,
        admin_link = if user.role == "admin" {
            r#"<a href="/admin" class="btn btn-primary">Admin Panel</a>"#
        } else {
            ""
        },
    )
}

pub async fn handler(State(state): State<AppState>, req: Request) -> Html<String> {
    let user = req.extensions().get::<AuthUser>().cloned();
    let (recent_posts, total) = {
        let posts = state.posts.lock().expect("posts lock");
        (posts.clone(), posts.len())
    };

    let body = match &user {
        Some(u) => render(u, &recent_posts, total),
        None => "<p>Not authenticated</p>".to_string(),
    };

    Html(crate::pages::_layout::render_layout(
        "Dashboard",
        &body,
        user.as_ref(),
        None,
    ))
}

mod auth {
    pub fn require_auth() {}
}
