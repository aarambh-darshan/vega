use crate::app::{current_user_from_headers, AppState};
use vega::server::web::{Html, Request, State};

#[vega::page(mode = "ssg")]
pub fn DocsIndex() -> &'static str {
    "Docs"
}

pub fn render() -> String {
    r#"<h1>Documentation</h1>
    <p>Quick reference for building Vega applications.</p>

    <div class="grid grid-2" style="margin-top:1.5rem">
        <div class="card">
            <h3>📁 File-Based Routing</h3>
            <pre><code>pages/
├── index.rs        → GET /
├── about.rs        → GET /about
├── blog/
│   ├── index.rs    → GET /blog
│   └── [slug].rs   → GET /blog/:slug
├── docs/
│   └── [...path].rs → GET /docs/*path
└── (auth)/
    ├── login.rs    → GET /login
    └── register.rs → GET /register</code></pre>
        </div>
        <div class="card">
            <h3>🔌 API Routes</h3>
            <pre><code>api/
├── auth/
│   ├── login.rs    → POST /api/auth/login
│   └── register.rs → POST /api/auth/register
├── posts/
│   ├── index.rs    → GET /api/posts
│   └── [id].rs     → GET|PUT|DELETE /api/posts/:id
└── _vega/
    └── routes.rs   → GET /api/_vega/routes</code></pre>
        </div>
        <div class="card">
            <h3>📄 Page Macros</h3>
            <pre><code>#[vega::page(mode = "ssr")]
pub fn MyPage() -> &'static str {
    "Server-rendered page"
}

#[vega::page(mode = "ssg")]
pub fn StaticPage() -> &'static str {
    "Built at compile time"
}

#[vega::page(mode = "csr",
  middleware = [auth::require_auth])]
pub fn ProtectedPage() -> &'static str {
    "Auth required"
}</code></pre>
        </div>
        <div class="card">
            <h3>🔌 API Macros</h3>
            <pre><code>#[vega::get]
pub fn ListPosts() -> &'static str {
    "GET handler"
}

#[vega::post(middleware = [auth::require_auth])]
pub fn CreatePost() -> &'static str {
    "POST handler"
}

#[vega::delete(middleware = [auth::require_admin])]
pub fn DeletePost() -> &'static str {
    "DELETE handler"
}</code></pre>
        </div>
    </div>

    <p style="margin-top:1.5rem">Try <a href="/docs/routing/dynamic">/docs/routing/dynamic</a> to test catch-all params.</p>"#.to_string()
}

pub async fn handler(State(state): State<AppState>, req: Request) -> Html<String> {
    let user = current_user_from_headers(req.headers(), &state).await;
    let body = render();
    Html(crate::pages::_layout::render_layout(
        "Docs",
        &body,
        user.as_ref(),
        None,
    ))
}
