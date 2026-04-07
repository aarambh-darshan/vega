use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use vega::config::VegaConfig;
use vega::core::RouteManifest;
use vega::server::{
    parse_cookie_map, web::HeaderMap, AuthService, AuthUser, InMemoryAuthService,
    InMemorySessionStore, SessionStore,
};

#[derive(Clone)]
pub struct AppState {
    pub(crate) auth: InMemoryAuthService,
    pub(crate) sessions: InMemorySessionStore,
    pub(crate) posts: Arc<Mutex<Vec<Post>>>,
    pub(crate) manifest: RouteManifest,
    pub(crate) secure_cookie: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: u64,
    pub slug: String,
    pub title: String,
    pub excerpt: String,
    pub body: String,
    pub tags: Vec<String>,
    pub author_email: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BlogQuery {
    pub q: Option<String>,
    pub tag: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub sort: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IdParam {
    pub id: u64,
}

/// Convention entry point — called by the auto-generated main.
pub async fn create_state(
    manifest: RouteManifest,
    config: &VegaConfig,
) -> anyhow::Result<AppState> {
    let auth = InMemoryAuthService::default();
    let sessions = InMemorySessionStore::default();

    auth.register("admin@vega.dev", "admin123", "admin").await?;
    auth.register("member@vega.dev", "member123", "member")
        .await?;

    Ok(AppState {
        auth,
        sessions,
        posts: Arc::new(Mutex::new(seed_posts())),
        manifest,
        secure_cookie: config.app.base_url.starts_with("https://"),
    })
}

pub async fn current_user_from_headers(headers: &HeaderMap, state: &AppState) -> Option<AuthUser> {
    let cookies = parse_cookie_map(headers);
    let token = cookies.get("vega_session")?;
    state.sessions.get_session(token).await.ok().flatten()
}

pub fn filter_posts(state: &AppState, query: &BlogQuery) -> Vec<Post> {
    let mut posts = state.posts.lock().expect("posts lock").clone();

    if let Some(q) = &query.q {
        let q = q.to_ascii_lowercase();
        posts.retain(|post| {
            post.title.to_ascii_lowercase().contains(&q)
                || post.excerpt.to_ascii_lowercase().contains(&q)
                || post.body.to_ascii_lowercase().contains(&q)
        });
    }

    if let Some(tag) = &query.tag {
        let tag = tag.to_ascii_lowercase();
        posts.retain(|post| {
            post.tags
                .iter()
                .any(|value| value.to_ascii_lowercase() == tag)
        });
    }

    match query.sort.as_deref() {
        Some("old") => posts.sort_by(|a, b| a.id.cmp(&b.id)),
        _ => posts.sort_by(|a, b| b.id.cmp(&a.id)),
    }

    posts
}

pub fn seed_posts() -> Vec<Post> {
    vec![
        Post {
            id: 1,
            slug: "vega-routing-basics".to_string(),
            title: "Vega Routing Basics".to_string(),
            excerpt: "File-based routes, dynamic params, and catch-all segments.".to_string(),
            body: "Vega scans your pages/ directory at build time and generates route registration code. Each .rs file becomes a route: index.rs maps to /, about.rs maps to /about, and blog/[slug].rs maps to /blog/:slug. Dynamic segments use bracket notation, and catch-all segments use triple-dot notation like [...path].rs. Route groups with parentheses like (auth)/ share layouts without adding URL segments.".to_string(),
            tags: vec!["rust".to_string(), "routing".to_string()],
            author_email: "admin@vega.dev".to_string(),
        },
        Post {
            id: 2,
            slug: "middleware-order-in-vega".to_string(),
            title: "Middleware Order in Vega".to_string(),
            excerpt: "Global, directory, and route middleware ordering explained.".to_string(),
            body: "Vega applies middleware in a deterministic order: global middleware (from your runtime setup) runs first, then directory-level middleware (from _layout.rs files), and finally route-level middleware (from the #[page(middleware = [...])] attribute). This ensures auth checks, logging, and rate limiting always execute in the expected order, avoiding subtle bugs in production.".to_string(),
            tags: vec!["middleware".to_string(), "security".to_string()],
            author_email: "admin@vega.dev".to_string(),
        },
        Post {
            id: 3,
            slug: "typed-query-params".to_string(),
            title: "Typed Query Parameters".to_string(),
            excerpt: "Decode ?page=2&q=rust into typed Rust structs automatically.".to_string(),
            body: "Vega's query helpers parse URL search parameters into typed Rust structs using serde. Define a struct with Optional fields, derive Deserialize, and call serde_urlencoded::from_str() on the query string. The framework handles URL decoding, type conversion, and default values. Combined with Axum's extractor pattern, you get compile-time guarantees on your query parameter handling.".to_string(),
            tags: vec!["rust".to_string(), "query".to_string()],
            author_email: "member@vega.dev".to_string(),
        },
        Post {
            id: 4,
            slug: "ssg-and-isr-planning".to_string(),
            title: "SSG and ISR Planning".to_string(),
            excerpt: "Static generation today, incremental static regeneration next.".to_string(),
            body: "Vega supports SSG (Static Site Generation) where pages are rendered to HTML at build time. The vega build command walks all SSG-annotated pages and writes .html files to the dist/ directory. ISR (Incremental Static Regeneration) is planned for v0.9+, where static pages can be revalidated in the background after a configurable time interval, combining the performance of SSG with the freshness of SSR.".to_string(),
            tags: vec!["ssg".to_string(), "isr".to_string()],
            author_email: "admin@vega.dev".to_string(),
        },
        Post {
            id: 5,
            slug: "api-routes-and-server-functions".to_string(),
            title: "API Routes and Server Functions".to_string(),
            excerpt: "Build type-safe REST APIs with co-located route files.".to_string(),
            body: "Files in the api/ directory become API endpoints automatically. Use #[vega::get], #[vega::post], #[vega::put], #[vega::delete] macros to define HTTP method handlers. Each handler receives typed access to path params, query strings, headers, cookies, and request bodies. Responses use ApiResponse::json() for success and ApiError::not_found() etc. for errors — all with proper HTTP status codes.".to_string(),
            tags: vec!["api".to_string(), "rust".to_string()],
            author_email: "admin@vega.dev".to_string(),
        },
        Post {
            id: 6,
            slug: "session-auth-deep-dive".to_string(),
            title: "Session Auth Deep Dive".to_string(),
            excerpt: "Cookie-based sessions with HttpOnly, SameSite, and role guards.".to_string(),
            body: "Vega's auth system uses HttpOnly cookies with SameSite=Lax for session management. When a user logs in, a UUID session token is generated and stored server-side. The token is sent to the browser as a cookie that JavaScript cannot access (HttpOnly). Middleware functions like require_auth and require_admin extract the cookie, look up the session, and either inject the user into the request or return a 401/403 response.".to_string(),
            tags: vec!["auth".to_string(), "security".to_string()],
            author_email: "member@vega.dev".to_string(),
        },
    ]
}

pub fn unique_slug(posts: &[Post], title: &str, id: u64) -> String {
    let base = slugify(title);
    if posts.iter().all(|post| post.slug != base) {
        return base;
    }
    format!("{base}-{id}")
}

pub fn slugify(input: &str) -> String {
    let mut value = input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();

    while value.contains("--") {
        value = value.replace("--", "-");
    }

    value.trim_matches('-').to_string()
}

pub fn esc(input: &str) -> String {
    vega::core::html_escape(input)
}
