//! # vega-core
//!
//! Core types, enums, and data structures for the Vega web framework.
//!
//! This crate defines the fundamental building blocks shared across all Vega crates:
//! - [`RenderMode`] — SSR, SSG, CSR, ISR page render modes
//! - [`HttpMethod`] — HTTP method enum for API routes
//! - [`SegmentKind`] — file-based routing segment classification
//! - [`PageMeta`] / [`ApiMeta`] — metadata attached by proc macros
//! - [`RouteEntry`] / [`ApiRouteEntry`] — discovered route descriptors
//! - [`RouteManifest`] — complete route manifest for a Vega project

use serde::{Deserialize, Serialize};
use std::fmt;

/// Render mode for a Vega page.
///
/// Each page declares its render mode via `#[vega::page(mode = "...")]`.
/// The mode determines how and when the page HTML is generated.
///
/// # Variants
///
/// - `Ssr` — Server-Side Rendering: HTML rendered per request on the server
/// - `Ssg` — Static Site Generation: HTML rendered at build time
/// - `Csr` — Client-Side Rendering: empty shell served, WASM renders in browser
/// - `Isr` — Incremental Static Regeneration: static with background revalidation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RenderMode {
    /// Server-Side Rendering — HTML generated per request.
    #[default]
    Ssr,
    /// Static Site Generation — HTML generated at build time.
    Ssg,
    /// Client-Side Rendering — empty HTML shell, WASM renders in browser.
    Csr,
    /// Incremental Static Regeneration — static with timed revalidation.
    Isr,
}

impl RenderMode {
    /// Parse a render mode from a string literal (case-insensitive).
    ///
    /// # Examples
    /// ```
    /// use vega_core::RenderMode;
    /// assert_eq!(RenderMode::from_literal("SSR"), Some(RenderMode::Ssr));
    /// assert_eq!(RenderMode::from_literal("csr"), Some(RenderMode::Csr));
    /// assert_eq!(RenderMode::from_literal("unknown"), None);
    /// ```
    pub fn from_literal(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "ssr" => Some(Self::Ssr),
            "ssg" => Some(Self::Ssg),
            "csr" => Some(Self::Csr),
            "isr" => Some(Self::Isr),
            _ => None,
        }
    }

    /// Return the render mode as a lowercase string slice.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ssr => "ssr",
            Self::Ssg => "ssg",
            Self::Csr => "csr",
            Self::Isr => "isr",
        }
    }

    /// Returns `true` if the page is rendered on the server (SSR or SSG).
    pub fn is_server_rendered(self) -> bool {
        matches!(self, Self::Ssr | Self::Ssg | Self::Isr)
    }

    /// Returns `true` if the page is rendered entirely in the browser.
    pub fn is_client_only(self) -> bool {
        matches!(self, Self::Csr)
    }
}

impl fmt::Display for RenderMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// HTTP method for API route handlers.
///
/// Used by `#[vega::get]`, `#[vega::post]`, etc. macros to tag API endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl HttpMethod {
    /// Return the HTTP method as an uppercase string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Classification of a file-based routing segment.
///
/// Vega's file scanner examines each filename and directory name in `pages/` and `api/`
/// to determine the type of route segment it represents.
///
/// # Naming Rules
///
/// | Filename | Segment Kind | URL Effect |
/// |----------|-------------|------------|
/// | `index.rs` | `Index` | Maps to parent path |
/// | `about.rs` | `Static("about")` | `/about` |
/// | `[slug].rs` | `Dynamic("slug")` | `/:slug` |
/// | `[...path].rs` | `CatchAll("path")` | `/*path` |
/// | `(auth)/` | `Group("auth")` | No URL segment |
/// | `_layout.rs` | `Layout` | Wraps children |
/// | `_error.rs` | `Error` | Error boundary |
/// | `_loading.rs` | `Loading` | Loading placeholder |
/// | `_not_found.rs` | `NotFound` | 404 handler |
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentKind {
    /// A static named segment: `about.rs` → `/about`.
    Static(String),
    /// Index file: `index.rs` → maps to parent directory path.
    Index,
    /// Dynamic parameter: `[slug].rs` → `/:slug`.
    Dynamic(String),
    /// Catch-all parameter: `[...path].rs` → `/*path`.
    CatchAll(String),
    /// Route group: `(auth)/` → no URL segment, shares layout.
    Group(String),
    /// Layout wrapper: `_layout.rs`.
    Layout,
    /// Error boundary: `_error.rs`.
    Error,
    /// Loading placeholder: `_loading.rs`.
    Loading,
    /// Not-found handler: `_not_found.rs`.
    NotFound,
}

impl SegmentKind {
    /// Returns `true` if this segment is a special file (`_layout`, `_error`, `_loading`, `_not_found`).
    pub fn is_special_file(&self) -> bool {
        matches!(
            self,
            Self::Layout | Self::Error | Self::Loading | Self::NotFound
        )
    }

    /// Returns `true` if this segment contributes a URL path component.
    pub fn has_url_segment(&self) -> bool {
        matches!(self, Self::Static(_) | Self::Dynamic(_) | Self::CatchAll(_))
    }
}

impl fmt::Display for SegmentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static(s) => write!(f, "{s}"),
            Self::Index => write!(f, "index"),
            Self::Dynamic(s) => write!(f, "[{s}]"),
            Self::CatchAll(s) => write!(f, "[...{s}]"),
            Self::Group(s) => write!(f, "({s})"),
            Self::Layout => write!(f, "_layout"),
            Self::Error => write!(f, "_error"),
            Self::Loading => write!(f, "_loading"),
            Self::NotFound => write!(f, "_not_found"),
        }
    }
}

/// Metadata emitted by the `#[vega::page]` proc macro.
///
/// This constant is generated alongside every page component and is read
/// by the build-time codegen to determine render mode and middleware.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageMeta {
    /// Render mode for this page (SSR, SSG, CSR, ISR).
    pub mode: RenderMode,
    /// ISR revalidation interval in seconds (only for `mode = "isr"`).
    pub revalidate: Option<u64>,
    /// Middleware function paths to apply to this page's route.
    pub middleware: &'static [&'static str],
    /// Component function name (e.g., `"BlogPost"`).
    pub component_name: &'static str,
    /// Source file path via `file!()`.
    pub file: &'static str,
}

impl Default for PageMeta {
    fn default() -> Self {
        Self {
            mode: RenderMode::Ssr,
            revalidate: None,
            middleware: &[],
            component_name: "Page",
            file: "unknown",
        }
    }
}

/// Metadata emitted by API method macros (`#[get]`, `#[post]`, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiMeta {
    /// HTTP method this handler responds to.
    pub method: HttpMethod,
    /// Middleware function paths to apply to this API endpoint.
    pub middleware: &'static [&'static str],
    /// Handler function name.
    pub fn_name: &'static str,
    /// Source file path via `file!()`.
    pub file: &'static str,
}

/// A discovered page route entry from the `pages/` directory scanner.
///
/// Produced by `vega-router` at compile time and included in the [`RouteManifest`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteEntry {
    /// Relative file path from `pages/` root (e.g., `"blog/[slug].rs"`).
    pub file_path: String,
    /// URL route path with parameter markers (e.g., `"/blog/:slug"`).
    pub route_path: String,
    /// Rust module path (e.g., `"pages::blog::slug_dynamic"`).
    pub module_path: String,
    /// `true` if this is a special file (`_layout`, `_error`, etc.).
    pub is_special: bool,
    /// Layout module paths that wrap this route, from outermost to innermost.
    pub layouts: Vec<String>,
}

/// A discovered API route entry from the `api/` directory scanner.
///
/// Produced by `vega-router` at compile time and included in the [`RouteManifest`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiRouteEntry {
    /// Relative file path from `api/` root.
    pub file_path: String,
    /// URL route path with `/api` prefix (e.g., `"/api/users/:id"`).
    pub route_path: String,
    /// Rust module path (e.g., `"api::users::id_dynamic"`).
    pub module_path: String,
    /// HTTP methods this endpoint responds to.
    pub methods: Vec<HttpMethod>,
}

/// Complete route manifest for a Vega project.
///
/// Contains all discovered page routes and API routes. This is the primary
/// data structure passed from build-time codegen to the runtime server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RouteManifest {
    /// All discovered page routes.
    pub pages: Vec<RouteEntry>,
    /// All discovered API routes.
    pub api: Vec<ApiRouteEntry>,
}

impl RouteManifest {
    /// Returns the total number of routes (pages + API).
    pub fn total_routes(&self) -> usize {
        self.pages.iter().filter(|p| !p.is_special).count() + self.api.len()
    }
}

/// Parameters for a statically generated page path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SsgPath {
    /// The route pattern (e.g., `"/blog/:slug"`).
    pub route: String,
    /// Parameter values as JSON (e.g., `{"slug": "hello-world"}`).
    pub params: serde_json::Value,
}

/// Placeholder configuration for ISR mode (v0.9+).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IsrConfig {
    /// Seconds between background revalidations.
    pub revalidate_seconds: u64,
}

/// Edge deployment target (v0.9+ planned).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeTarget {
    /// Deploy to Cloudflare Workers.
    Cloudflare,
    /// Deploy to Fly.io.
    FlyIo,
}

/// V1 stability checklist placeholder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct V1Checklist {
    /// All public APIs finalized with no breaking changes.
    pub stabilize_public_api: bool,
    /// Performance benchmark suite passing.
    pub benchmark_suite: bool,
    /// Security audit completed.
    pub security_audit: bool,
}

/// Escape an HTML string to prevent XSS.
///
/// Replaces `&`, `<`, `>`, `"`, and `'` with their HTML entity equivalents.
///
/// # Examples
/// ```
/// assert_eq!(vega_core::html_escape("<script>alert('xss')</script>"),
///            "&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;");
/// ```
pub fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_render_mode() {
        assert_eq!(RenderMode::from_literal("SSR"), Some(RenderMode::Ssr));
        assert_eq!(RenderMode::from_literal("ssg"), Some(RenderMode::Ssg));
        assert_eq!(RenderMode::from_literal("csr"), Some(RenderMode::Csr));
        assert_eq!(RenderMode::from_literal("isr"), Some(RenderMode::Isr));
        assert_eq!(RenderMode::from_literal("bad"), None);
    }

    #[test]
    fn render_mode_display() {
        assert_eq!(RenderMode::Ssr.to_string(), "ssr");
        assert_eq!(RenderMode::Csr.to_string(), "csr");
    }

    #[test]
    fn render_mode_predicates() {
        assert!(RenderMode::Ssr.is_server_rendered());
        assert!(RenderMode::Ssg.is_server_rendered());
        assert!(!RenderMode::Csr.is_server_rendered());
        assert!(RenderMode::Csr.is_client_only());
    }

    #[test]
    fn method_string() {
        assert_eq!(HttpMethod::Delete.as_str(), "DELETE");
        assert_eq!(HttpMethod::Get.to_string(), "GET");
    }

    #[test]
    fn segment_display() {
        assert_eq!(SegmentKind::Dynamic("slug".into()).to_string(), "[slug]");
        assert_eq!(
            SegmentKind::CatchAll("path".into()).to_string(),
            "[...path]"
        );
        assert_eq!(SegmentKind::Group("auth".into()).to_string(), "(auth)");
    }

    #[test]
    fn default_page_meta() {
        let meta = PageMeta::default();
        assert_eq!(meta.mode, RenderMode::Ssr);
        assert_eq!(meta.middleware.len(), 0);
    }

    #[test]
    fn manifest_count() {
        let manifest = RouteManifest {
            pages: vec![
                RouteEntry {
                    file_path: "index.rs".into(),
                    route_path: "/".into(),
                    module_path: "pages::index".into(),
                    is_special: false,
                    layouts: vec![],
                },
                RouteEntry {
                    file_path: "_layout.rs".into(),
                    route_path: "".into(),
                    module_path: "pages::_layout".into(),
                    is_special: true,
                    layouts: vec![],
                },
            ],
            api: vec![ApiRouteEntry {
                file_path: "hello.rs".into(),
                route_path: "/api/hello".into(),
                module_path: "api::hello".into(),
                methods: vec![HttpMethod::Get],
            }],
        };
        assert_eq!(manifest.total_routes(), 2); // 1 page + 1 api
    }

    #[test]
    fn html_escape_works() {
        assert_eq!(
            html_escape("<b>\"hi\"</b>"),
            "&lt;b&gt;&quot;hi&quot;&lt;/b&gt;"
        );
    }
}
