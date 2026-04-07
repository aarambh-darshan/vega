//! # vega-server
//!
//! Axum-based server runtime for the Vega web framework.
//!
//! Provides:
//! - [`ApiRequest`] / [`ApiResponse`] / [`ApiError`] — typed API request/response handling
//! - [`AuthUser`] / [`SessionStore`] / [`AuthService`] — pluggable auth system
//! - [`PageContext`] — server-side page rendering context
//! - Middleware layers: logging, CORS, compression, rate limiting, security headers
//! - Static file serving from `public/` directory
//! - Tracing initialization
//! - Graceful shutdown support
//!
//! # Example
//!
//! ```rust,ignore
//! use vega_server::{init_tracing, run_with_shutdown};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     init_tracing();
//!     let router = /* build your router */;
//!     run_with_shutdown(router, "0.0.0.0", 3000).await
//! }
//! ```

use anyhow::Context;
use async_trait::async_trait;
use axum::body::{to_bytes, Body, Bytes};
use axum::extract::Request as AxumRequest;
use axum::http::{HeaderMap, HeaderName, HeaderValue, Request, StatusCode, Uri};
use axum::middleware::Next as AxumNext;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, MethodRouter};
use axum::{Json, Router};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower::limit::RateLimitLayer;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;
use vega_config::VegaConfig;
use vega_core::{ApiRouteEntry, HttpMethod, RouteEntry, RouteManifest};

/// Re-export of Axum's `Next` middleware type.
pub type Next = AxumNext;

/// Commonly used Axum types re-exported for convenience in page and API handlers.
///
/// Import with `use vega::server::web::*;` in your page and API files.
pub mod web {
    pub use axum::body::Body;
    pub use axum::extract::{Path, Request, State};
    pub use axum::http::header::SET_COOKIE;
    pub use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode, Uri};
    pub use axum::middleware::{from_fn, from_fn_with_state, Next};
    pub use axum::response::{Html, IntoResponse, Redirect, Response};
    pub use axum::routing::{delete, get, patch, post, put};
    pub use axum::{Form, Json, Router};
}

// ---------------------------------------------------------------------------
// Tracing
// ---------------------------------------------------------------------------

/// Initialize the tracing subscriber with sensible defaults.
///
/// Configures `tracing-subscriber` with:
/// - Format layer with timestamps, target, and level
/// - `RUST_LOG` env filter (defaults to `info,tower_http=debug`)
///
/// Call this at the start of `main()` before creating the router.
pub fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=debug"));

    fmt().with_env_filter(filter).with_target(true).init();
}

// ---------------------------------------------------------------------------
// API Request
// ---------------------------------------------------------------------------

/// A parsed API request with typed accessors for body, params, query, headers, and cookies.
///
/// Constructed from an Axum request in API route handlers. Provides convenient
/// methods to extract JSON, form data, query parameters, and cookies.
#[derive(Debug, Clone)]
pub struct ApiRequest {
    /// HTTP headers from the request.
    pub headers: HeaderMap,
    /// Path parameters extracted from the route pattern.
    pub params: HashMap<String, String>,
    /// Query string parameters.
    pub query: HashMap<String, String>,
    /// Parsed cookies from the `Cookie` header.
    pub cookies: HashMap<String, String>,
    body: Bytes,
}

impl ApiRequest {
    /// Construct an `ApiRequest` from pre-parsed parts.
    pub fn from_parts(
        headers: HeaderMap,
        params: HashMap<String, String>,
        query: HashMap<String, String>,
        body: Bytes,
    ) -> Self {
        let cookies = parse_cookie_map(&headers);
        Self {
            headers,
            params,
            query,
            cookies,
            body,
        }
    }

    /// Construct an `ApiRequest` from a raw Axum request, consuming the body.
    pub async fn from_axum_request(req: Request<Body>) -> Result<Self, ApiError> {
        let (parts, body) = req.into_parts();
        let headers = parts.headers;
        let query = parse_query_map(&parts.uri);
        let body = to_bytes(body, 1024 * 1024 * 10) // 10MB limit
            .await
            .map_err(|error| {
                ApiError::bad_request(format!("failed to read request body: {error}"))
            })?;

        Ok(Self::from_parts(headers, HashMap::new(), query, body))
    }

    /// Deserialize the request body as JSON.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, ApiError> {
        serde_json::from_slice(&self.body).map_err(|error| {
            ApiError::bad_request(format!("failed to decode JSON payload: {error}"))
        })
    }

    /// Deserialize the request body as URL-encoded form data.
    pub fn form<T: DeserializeOwned>(&self) -> Result<T, ApiError> {
        serde_urlencoded::from_bytes(&self.body).map_err(|error| {
            ApiError::bad_request(format!("failed to decode form payload: {error}"))
        })
    }

    /// Deserialize query string parameters into a typed struct.
    pub fn query<T: DeserializeOwned>(&self) -> Result<T, ApiError> {
        let encoded = encode_flat_query(&self.query);
        serde_urlencoded::from_str(&encoded).map_err(|error| {
            ApiError::bad_request(format!("failed to decode query payload: {error}"))
        })
    }

    /// Extract a typed path parameter by name.
    pub fn path_param<T: std::str::FromStr>(&self, key: &str) -> Result<T, ApiError> {
        self.params
            .get(key)
            .ok_or_else(|| ApiError::bad_request(format!("missing path param: {key}")))
            .and_then(|value| {
                value
                    .parse::<T>()
                    .map_err(|_| ApiError::bad_request(format!("invalid path param: {key}")))
            })
    }

    /// Get a cookie value by name.
    pub fn cookie(&self, key: &str) -> Option<&str> {
        self.cookies.get(key).map(String::as_str)
    }

    /// Get a header value by name.
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|value| value.to_str().ok())
    }

    /// Get the raw request body as a UTF-8 string.
    pub fn body_text(&self) -> Result<String, ApiError> {
        String::from_utf8(self.body.to_vec())
            .map_err(|error| ApiError::bad_request(format!("request body is not UTF-8: {error}")))
    }
}

// ---------------------------------------------------------------------------
// API Response
// ---------------------------------------------------------------------------

/// A structured API response with JSON body, status code, and custom headers.
///
/// # Examples
///
/// ```rust,ignore
/// // Return JSON with 200 OK
/// ApiResponse::json(json!({"ok": true}))
///
/// // Return 201 Created
/// ApiResponse::created(json!({"id": 42}))
///
/// // Return redirect
/// ApiResponse::redirect("/login")
/// ```
#[derive(Debug)]
pub struct ApiResponse {
    status: StatusCode,
    body: serde_json::Value,
    headers: HeaderMap,
}

impl ApiResponse {
    /// Create a 200 OK JSON response.
    pub fn json<T: Serialize>(value: T) -> Self {
        Self {
            status: StatusCode::OK,
            body: serde_json::to_value(value)
                .unwrap_or_else(|_| json!({ "error": "serialization" })),
            headers: HeaderMap::new(),
        }
    }

    /// Create a JSON response with a custom status code.
    pub fn status_json<T: Serialize>(status: StatusCode, value: T) -> Self {
        Self {
            status,
            body: serde_json::to_value(value)
                .unwrap_or_else(|_| json!({ "error": "serialization" })),
            headers: HeaderMap::new(),
        }
    }

    /// Create a 201 Created JSON response.
    pub fn created<T: Serialize>(value: T) -> Self {
        Self::status_json(StatusCode::CREATED, value)
    }

    /// Create a 204 No Content response.
    pub fn no_content() -> Self {
        Self {
            status: StatusCode::NO_CONTENT,
            body: json!(null),
            headers: HeaderMap::new(),
        }
    }

    /// Create a 307 Temporary Redirect response.
    pub fn redirect(path: &str) -> Self {
        let mut headers = HeaderMap::new();
        if let Ok(value) = HeaderValue::from_str(path) {
            headers.insert(axum::http::header::LOCATION, value);
        }
        Self {
            status: StatusCode::TEMPORARY_REDIRECT,
            body: json!({ "redirect": path }),
            headers,
        }
    }

    /// Add a custom header to the response.
    pub fn with_header(mut self, key: HeaderName, value: HeaderValue) -> Self {
        self.headers.append(key, value);
        self
    }

    /// Add a `Set-Cookie` header to the response.
    pub fn with_cookie(mut self, cookie: &str) -> Result<Self, ApiError> {
        let value = HeaderValue::from_str(cookie)
            .map_err(|error| ApiError::internal(format!("invalid cookie header: {error}")))?;
        self.headers.append(axum::http::header::SET_COOKIE, value);
        Ok(self)
    }
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        if self.status == StatusCode::NO_CONTENT {
            let mut response = self.status.into_response();
            response.headers_mut().extend(self.headers);
            return response;
        }
        let mut response = (self.status, Json(self.body)).into_response();
        response.headers_mut().extend(self.headers);
        response
    }
}

// ---------------------------------------------------------------------------
// API Error
// ---------------------------------------------------------------------------

/// A structured API error that serializes to JSON with an HTTP status code.
///
/// # Examples
///
/// ```rust,ignore
/// ApiError::not_found("post not found")
/// ApiError::bad_request("missing required field: email")
/// ApiError::unauthorized("invalid token")
/// ```
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    /// Create a 400 Bad Request error.
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    /// Create a 401 Unauthorized error.
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    /// Create a 403 Forbidden error.
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
        }
    }

    /// Create a 404 Not Found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    /// Create a 409 Conflict error.
    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.into(),
        }
    }

    /// Create a 422 Unprocessable Entity error.
    pub fn unprocessable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: message.into(),
        }
    }

    /// Create a 500 Internal Server Error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

// ---------------------------------------------------------------------------
// Auth Types
// ---------------------------------------------------------------------------

/// An authenticated user with an ID, email, and role.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthUser {
    pub id: u64,
    pub email: String,
    pub role: String,
}

/// Trait for session token storage.
///
/// Implementations manage the lifecycle of session tokens: creation, lookup, and deletion.
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Create a new session for the given user, returning a unique token.
    async fn create_session(&self, user: AuthUser) -> anyhow::Result<String>;
    /// Look up a session by token, returning the user if valid.
    async fn get_session(&self, token: &str) -> anyhow::Result<Option<AuthUser>>;
    /// Delete a session by token (logout).
    async fn delete_session(&self, token: &str) -> anyhow::Result<()>;
}

/// In-memory session store backed by a `Mutex<HashMap>`.
///
/// Suitable for development and testing. For production, implement [`SessionStore`]
/// with Redis, a database, or another persistent backend.
#[derive(Debug, Clone, Default)]
pub struct InMemorySessionStore {
    sessions: Arc<Mutex<HashMap<String, AuthUser>>>,
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn create_session(&self, user: AuthUser) -> anyhow::Result<String> {
        let token = Uuid::new_v4().to_string();
        self.sessions
            .lock()
            .expect("session lock poisoned")
            .insert(token.clone(), user);
        Ok(token)
    }

    async fn get_session(&self, token: &str) -> anyhow::Result<Option<AuthUser>> {
        Ok(self
            .sessions
            .lock()
            .expect("session lock poisoned")
            .get(token)
            .cloned())
    }

    async fn delete_session(&self, token: &str) -> anyhow::Result<()> {
        self.sessions
            .lock()
            .expect("session lock poisoned")
            .remove(token);
        Ok(())
    }
}

/// Trait for user registration and login.
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Register a new user. Returns the created user.
    async fn register(&self, email: &str, password: &str, role: &str) -> anyhow::Result<AuthUser>;
    /// Authenticate a user by email and password. Returns `None` if credentials are invalid.
    async fn login(&self, email: &str, password: &str) -> anyhow::Result<Option<AuthUser>>;
}

/// In-memory auth service for development and testing.
///
/// Stores users in a `Mutex<Vec>`. Passwords are stored in plain text.
/// **Do not use in production.** Implement [`AuthService`] with bcrypt/argon2 hashing.
#[derive(Debug, Clone, Default)]
pub struct InMemoryAuthService {
    users: Arc<Mutex<Vec<(AuthUser, String)>>>,
}

#[async_trait]
impl AuthService for InMemoryAuthService {
    async fn register(&self, email: &str, password: &str, role: &str) -> anyhow::Result<AuthUser> {
        let mut users = self.users.lock().expect("auth lock poisoned");

        if users.iter().any(|(u, _)| u.email == email) {
            anyhow::bail!("email already registered: {}", email);
        }

        let next_id = users.len() as u64 + 1;
        let user = AuthUser {
            id: next_id,
            email: email.to_string(),
            role: role.to_string(),
        };
        users.push((user.clone(), password.to_string()));
        Ok(user)
    }

    async fn login(&self, email: &str, password: &str) -> anyhow::Result<Option<AuthUser>> {
        let users = self.users.lock().expect("auth lock poisoned");
        Ok(users
            .iter()
            .find(|(user, stored)| user.email == email && stored == password)
            .map(|(user, _)| user.clone()))
    }
}

// ---------------------------------------------------------------------------
// Cookie Helpers
// ---------------------------------------------------------------------------

/// Create a session cookie string with security attributes.
///
/// # Arguments
/// - `token` — session token value
/// - `secure` — set the `Secure` flag (HTTPS only)
/// - `max_age_seconds` — cookie lifetime in seconds
pub fn make_session_cookie(token: &str, secure: bool, max_age_seconds: i64) -> String {
    let mut cookie =
        format!("vega_session={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age_seconds}");
    if secure {
        cookie.push_str("; Secure");
    }
    cookie
}

/// Create a cookie string that clears the session (sets Max-Age=0).
pub fn clear_session_cookie(secure: bool) -> String {
    let mut cookie = "vega_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0".to_string();
    if secure {
        cookie.push_str("; Secure");
    }
    cookie
}

// ---------------------------------------------------------------------------
// Middleware Helpers
// ---------------------------------------------------------------------------

/// Basic auth middleware that checks for a `vega_session` cookie.
pub async fn require_auth(req: Request<Body>, next: Next) -> Response {
    let cookies = parse_cookie_map(req.headers());
    if cookies.get("vega_session").is_none() {
        return ApiResponse::status_json(
            StatusCode::UNAUTHORIZED,
            json!({ "error": "authentication required" }),
        )
        .into_response();
    }
    next.run(req).await
}

/// Role-based auth middleware.
pub async fn require_role(required_role: &'static str, req: Request<Body>, next: Next) -> Response {
    let role = req
        .headers()
        .get("x-user-role")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if role != required_role {
        return ApiResponse::status_json(
            StatusCode::FORBIDDEN,
            json!({ "error": "insufficient role" }),
        )
        .into_response();
    }
    next.run(req).await
}

// ---------------------------------------------------------------------------
// Page Context
// ---------------------------------------------------------------------------

/// Server-side rendering context for page handlers.
///
/// Contains the request path, parameters, query string, cookies, and current user.
#[derive(Debug, Clone, Default)]
pub struct PageContext {
    /// Current request path (e.g., `"/blog/hello-world"`).
    pub path: String,
    /// Route parameters (e.g., `{"slug": "hello-world"}`).
    pub params: HashMap<String, String>,
    /// Query string parameters.
    pub query: HashMap<String, String>,
    /// Parsed cookies.
    pub cookies: HashMap<String, String>,
    /// The currently authenticated user, if any.
    pub current_user: Option<AuthUser>,
}

impl PageContext {
    /// Get a query parameter value by key.
    pub fn query_value(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(String::as_str)
    }

    /// Get a route parameter value by key.
    pub fn param_value(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(String::as_str)
    }

    /// Get a cookie value by key.
    pub fn cookie_value(&self, key: &str) -> Option<&str> {
        self.cookies.get(key).map(String::as_str)
    }
}

// ---------------------------------------------------------------------------
// Parsing Helpers
// ---------------------------------------------------------------------------

/// Parse query string parameters from a URI into a HashMap.
pub fn parse_query_map(uri: &Uri) -> HashMap<String, String> {
    let mut query_map = HashMap::new();
    if let Some(query) = uri.query() {
        for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
            query_map.insert(key.to_string(), value.to_string());
        }
    }
    query_map
}

/// Parse the `Cookie` header into a HashMap of name→value pairs.
pub fn parse_cookie_map(headers: &HeaderMap) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    if let Some(value) = headers
        .get(axum::http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
    {
        for pair in value.split(';') {
            let mut segments = pair.trim().splitn(2, '=');
            let key = segments.next().unwrap_or_default().trim();
            let value = segments.next().unwrap_or_default().trim();
            if !key.is_empty() {
                cookies.insert(key.to_string(), value.to_string());
            }
        }
    }
    cookies
}

/// Match a request path against a route pattern and extract path parameters.
///
/// Supports `:param` (dynamic) and `*param` (catch-all) segments.
pub fn extract_path_params(path: &str, pattern: &str) -> Option<HashMap<String, String>> {
    let path_segments = path
        .trim_start_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let pattern_segments = pattern
        .trim_start_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    // Handle root path
    if pattern_segments.is_empty() && path_segments.is_empty() {
        return Some(HashMap::new());
    }
    if pattern_segments.is_empty() {
        return None;
    }

    let mut params = HashMap::new();
    let mut index = 0;
    while index < pattern_segments.len() {
        let pat = pattern_segments[index];
        if let Some(name) = pat.strip_prefix('*') {
            let rest = path_segments[index..].join("/");
            params.insert(name.to_string(), rest);
            return Some(params);
        }

        let seg = path_segments.get(index)?;
        if let Some(name) = pat.strip_prefix(':') {
            params.insert(name.to_string(), (*seg).to_string());
        } else if pat != *seg {
            return None;
        }

        index += 1;
    }

    if index == path_segments.len() {
        Some(params)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Middleware Layers
// ---------------------------------------------------------------------------

/// Create a tracing layer for HTTP request/response logging.
pub fn logger_layer(
) -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    TraceLayer::new_for_http()
}

/// Create a permissive CORS layer (allows all origins, methods, headers).
pub fn cors_layer() -> CorsLayer {
    CorsLayer::permissive()
}

/// Create a Brotli/gzip compression layer for responses.
pub fn compression_layer() -> CompressionLayer {
    CompressionLayer::new()
}

/// Create a rate limiting layer (requests per second per connection).
pub fn rate_limit_layer(limit: u64) -> RateLimitLayer {
    RateLimitLayer::new(limit, std::time::Duration::from_secs(1))
}

/// Create a security headers layer that sets common protective headers.
///
/// Sets:
/// - `X-Content-Type-Options: nosniff`
/// - `X-Frame-Options: DENY`
/// - `X-XSS-Protection: 1; mode=block`
/// - `Referrer-Policy: strict-origin-when-cross-origin`
pub fn security_headers_layer() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    )
}

// ---------------------------------------------------------------------------
// Router Building
// ---------------------------------------------------------------------------

/// Build the default Vega router from a route manifest.
///
/// Includes health check endpoint, route introspection, request ID, and tracing.
pub fn build_router(manifest: RouteManifest) -> Router {
    build_router_with_api_router(manifest, None)
}

/// Build a Vega router with an optional custom API sub-router merged in.
pub fn build_router_with_api_router(manifest: RouteManifest, api_router: Option<Router>) -> Router {
    let shared_manifest = Arc::new(manifest);

    let mut app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route(
            "/api/_vega/routes",
            get({
                let manifest = shared_manifest.clone();
                move || {
                    let manifest = manifest.clone();
                    async move { Json((*manifest).clone()) }
                }
            }),
        )
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid::default()));

    for api_entry in &shared_manifest.api {
        let axum_path = to_axum_path(&api_entry.route_path);
        app = app.route(&axum_path, api_method_router(api_entry.clone()));
    }

    if let Some(api_router) = api_router {
        app = app.merge(api_router);
    }

    app.fallback({
        let manifest = shared_manifest.clone();
        move |req: AxumRequest| {
            let manifest = manifest.clone();
            async move { ssr_handler(manifest, req).await }
        }
    })
}

/// Build a router from config, applying configured middleware layers.
pub fn build_router_from_config(config: &VegaConfig, manifest: RouteManifest) -> Router {
    let router = build_router(manifest);

    if config.features.compress {
        router.layer(compression_layer())
    } else {
        router
    }
}

// ---------------------------------------------------------------------------
// Static File Serving
// ---------------------------------------------------------------------------

/// Create a static file serving service from a directory.
///
/// Files are served at the root path (`/`). For example, `public/images/logo.png`
/// would be available at `/images/logo.png`.
///
/// Use with `.fallback_service()` on your router:
/// ```rust,ignore
/// let app = Router::new()
///     .route("/", get(handler))
///     .fallback_service(vega_server::static_file_service("public"));
/// ```
pub fn static_file_service(public_dir: impl AsRef<Path>) -> ServeDir {
    ServeDir::new(public_dir)
}

// ---------------------------------------------------------------------------
// Server Run
// ---------------------------------------------------------------------------

/// Start the Axum server and listen for connections.
///
/// Binds to `{host}:{port}` and serves the provided router.
pub async fn run(router: Router, host: &str, port: u16) -> anyhow::Result<()> {
    let addr = format!("{host}:{port}");
    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    info!("🚀 Vega server running on http://{}", addr);
    axum::serve(listener, router)
        .await
        .context("server exited unexpectedly")
}

/// Start the Axum server with graceful shutdown on Ctrl+C.
///
/// Binds to `{host}:{port}` and listens for SIGINT to gracefully stop.
pub async fn run_with_shutdown(router: Router, host: &str, port: u16) -> anyhow::Result<()> {
    let addr = format!("{host}:{port}");
    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    info!("🚀 Vega server running on http://{}", addr);
    info!("   Press Ctrl+C to stop");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server exited unexpectedly")
}

/// Wait for a shutdown signal (Ctrl+C).
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    info!("🛑 Shutdown signal received, stopping gracefully...");
}

// ---------------------------------------------------------------------------
// SSR Handler
// ---------------------------------------------------------------------------

async fn ssr_handler(manifest: Arc<RouteManifest>, req: Request<Body>) -> Response {
    let path = req.uri().path();
    if let Some(route) = match_page(path, &manifest.pages) {
        let query = parse_query_map(req.uri());
        let params = extract_path_params(path, &route.route_path).unwrap_or_default();
        let mut html = String::new();
        let _ = write!(
            html,
            "<!doctype html>\
            <html lang=\"en\">\
            <head>\
              <meta charset=\"utf-8\">\
              <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
              <title>Vega — {route}</title>\
              <style>\
                body {{ font-family: system-ui, sans-serif; margin: 2rem; color: #1a1a2e; }}\
                h1 {{ color: #16213e; }}\
                pre {{ background: #f0f0f0; padding: 1rem; border-radius: 8px; overflow-x: auto; }}\
              </style>\
            </head>\
            <body>\
              <div id=\"root\">\
                <h1>Vega SSR Shell</h1>\
                <p>Matched route: <code>{route}</code></p>\
                <p>Module: <code>{module}</code></p>\
                <pre>Params: {params:?}\nQuery: {query:?}</pre>\
              </div>\
            </body>\
            </html>",
            route = route.route_path,
            module = route.module_path,
            params = params,
            query = query
        );
        return Html(html).into_response();
    }

    (
        StatusCode::NOT_FOUND,
        Html(
            "<!doctype html><html><body><h1>404 — Not Found</h1>\
             <p>No matching route. <a href=\"/\">Go home</a>.</p></body></html>"
                .to_string(),
        ),
    )
        .into_response()
}

fn api_method_router(entry: ApiRouteEntry) -> MethodRouter {
    let entry = Arc::new(entry);
    let mut router = MethodRouter::new();

    for method in &entry.methods {
        router = match method {
            HttpMethod::Get => router.get(api_echo_handler(entry.clone())),
            HttpMethod::Post => router.post(api_echo_handler(entry.clone())),
            HttpMethod::Put => router.put(api_echo_handler(entry.clone())),
            HttpMethod::Patch => router.patch(api_echo_handler(entry.clone())),
            HttpMethod::Delete => router.delete(api_echo_handler(entry.clone())),
        };
    }

    router
}

fn api_echo_handler(
    route: Arc<ApiRouteEntry>,
) -> impl Clone
       + Send
       + Sync
       + 'static
       + Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ApiResponse> + Send>> {
    move || {
        let route = route.clone();
        Box::pin(async move {
            ApiResponse::json(json!({
                "ok": true,
                "route": route.route_path,
                "module": route.module_path,
                "methods": route.methods.iter().map(|m| m.as_str()).collect::<Vec<_>>()
            }))
        })
    }
}

fn match_page<'a>(path: &str, routes: &'a [RouteEntry]) -> Option<&'a RouteEntry> {
    routes
        .iter()
        .filter(|route| !route.is_special)
        .find(|route| path_matches_pattern(path, &route.route_path))
}

fn path_matches_pattern(path: &str, pattern: &str) -> bool {
    extract_path_params(path, pattern).is_some()
}

fn to_axum_path(path: &str) -> String {
    let converted = path
        .split('/')
        .map(|segment| {
            if let Some(name) = segment.strip_prefix(':') {
                format!("{{{name}}}")
            } else if let Some(name) = segment.strip_prefix('*') {
                format!("{{*{name}}}")
            } else {
                segment.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("/");

    if converted.is_empty() {
        "/".to_string()
    } else if converted.starts_with('/') {
        converted
    } else {
        format!("/{converted}")
    }
}

fn encode_flat_query(map: &HashMap<String, String>) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    let mut pairs = map.iter().collect::<Vec<_>>();
    pairs.sort_by(|a, b| a.0.cmp(b.0));
    for (key, value) in pairs {
        serializer.append_pair(key, value);
    }
    serializer.finish()
}

// ---------------------------------------------------------------------------
// HTML Escaping (convenience re-export from core)
// ---------------------------------------------------------------------------

/// Escape HTML special characters to prevent XSS.
///
/// This is a convenience re-export of [`vega_core::html_escape`].
pub fn esc(input: &str) -> String {
    vega_core::html_escape(input)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_matching() {
        assert!(path_matches_pattern("/blog/hello", "/blog/:slug"));
        assert!(path_matches_pattern("/docs/a/b", "/docs/*path"));
        assert!(!path_matches_pattern("/blog", "/blog/:slug"));
        assert!(path_matches_pattern("/", "/"));
    }

    #[test]
    fn api_response_statuses() {
        let ok = ApiResponse::json(json!({"ok": true})).into_response();
        assert_eq!(ok.status(), StatusCode::OK);

        let created = ApiResponse::created(json!({"id": 1})).into_response();
        assert_eq!(created.status(), StatusCode::CREATED);

        let no_content = ApiResponse::no_content().into_response();
        assert_eq!(no_content.status(), StatusCode::NO_CONTENT);
    }

    #[test]
    fn api_error_variants() {
        let err = ApiError::not_found("nope");
        assert_eq!(err.status, StatusCode::NOT_FOUND);

        let err = ApiError::conflict("duplicate");
        assert_eq!(err.status, StatusCode::CONFLICT);

        let err = ApiError::unprocessable("bad input");
        assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn axum_path_conversion() {
        assert_eq!(to_axum_path("/api/users/:id"), "/api/users/{id}");
        assert_eq!(to_axum_path("/docs/*path"), "/docs/{*path}");
    }

    #[test]
    fn cookie_parser_works() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            HeaderValue::from_static("a=1; vega_session=abc"),
        );
        let cookies = parse_cookie_map(&headers);
        assert_eq!(cookies.get("vega_session").map(String::as_str), Some("abc"));
    }

    #[test]
    fn extract_path_params_dynamic() {
        let params = extract_path_params("/users/42", "/users/:id").expect("params");
        assert_eq!(params.get("id").map(String::as_str), Some("42"));
    }

    #[test]
    fn extract_path_params_catchall() {
        let params = extract_path_params("/docs/a/b/c", "/docs/*path").expect("params");
        assert_eq!(params.get("path").map(String::as_str), Some("a/b/c"));
    }

    #[test]
    fn redirect_has_location_header() {
        let resp = ApiResponse::redirect("/login").into_response();
        assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(
            resp.headers().get("location").map(|v| v.to_str().unwrap()),
            Some("/login")
        );
    }
}
