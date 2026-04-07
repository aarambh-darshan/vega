//! # vega
//!
//! **A Next.js-inspired full-stack web framework for Rust.**
//!
//! Vega (वेग — Sanskrit for *speed, momentum*) provides a structured layer on top of
//! Axum with file-based routing, SSR/SSG/CSR render modes, co-located API routes,
//! and zero-config conventions.
//!
//! # Architecture
//!
//! ```text
//! my-vega-app/
//! ├── pages/              # File-based routing (auto-scanned)
//! │   ├── index.rs        # → GET /
//! │   ├── about.rs        # → GET /about
//! │   └── blog/[slug].rs  # → GET /blog/:slug
//! ├── api/                # Co-located API routes (auto-scanned)
//! │   └── hello.rs        # → GET /api/hello
//! ├── components/         # Shared components (auto-scanned)
//! ├── sections/           # Landing page sections (auto-scanned)
//! ├── Vega.toml           # Framework configuration
//! └── build.rs            # Codegen entry point
//! ```
//!
//! # Quick Start
//!
//! ```rust,ignore
//! // pages/index.rs
//! #[vega::page(mode = "ssr")]
//! pub fn IndexPage() -> &'static str {
//!     "Hello from Vega!"
//! }
//! ```
//!
//! # Features
//!
//! - **File-based routing** — drop a `.rs` file in `pages/`, get a route
//! - **Render modes** — SSR, SSG, CSR, ISR per page
//! - **API routes** — files in `api/` become Axum handlers
//! - **Layouts** — `_layout.rs` wraps child routes automatically
//! - **Middleware** — Tower middleware via `#[page(middleware = [...])]`
//! - **Typed params** — `use_params`, `use_search_params`
//! - **CLI** — `vega new`, `vega dev`, `vega build`, `vega generate`

// Re-export proc macros at top level
pub use vega_macros::{delete, get, layout, page, patch, post, put, server_fn};

// Re-export client macro
pub use vega_client::vega_hydrate;

// Re-export data fetching at top level
pub use vega_fetch::{
    clear_search_params, decode_search_query, encode_search_query, fetch, merge_search_params,
    remove_search_param, search_params, set_search_params, use_mutation, use_params, use_query,
    use_search_params, FetchError, Mutation, ParamsError, QueryState, SearchParamHandle,
    SearchParamMode, SearchParamResult,
};

// Re-export commonly used server types at top level
pub use vega_server::{ApiError, ApiRequest, ApiResponse};

/// Client-side runtime and hydration utilities.
pub mod client {
    pub use vega_client::*;
}

/// Configuration parsing for `Vega.toml`.
pub mod config {
    pub use vega_config::*;
}

/// Core types, enums, and data structures.
pub mod core {
    pub use vega_core::*;
}

/// File-based route scanner and compile-time codegen.
pub mod router {
    pub use vega_router::*;
}

/// Axum server runtime, API types, auth, and middleware.
pub mod server {
    pub use vega_server::*;
}

/// Built-in middleware layers.
pub mod middleware {
    pub use vega_server::{
        compression_layer, cors_layer, logger_layer, rate_limit_layer, security_headers_layer,
    };
}

/// Data fetching and parameter helpers.
pub mod data {
    pub use vega_fetch::*;
}

/// Read a required environment variable, panicking if not set.
///
/// # Panics
///
/// Panics at runtime if the environment variable is not defined.
///
/// # Examples
///
/// ```rust,ignore
/// let db_url = vega::env!("DATABASE_URL");
/// ```
#[macro_export]
macro_rules! env {
    ($name:literal) => {{
        std::env::var($name)
            .unwrap_or_else(|_| panic!("missing required environment variable: {}", $name))
    }};
}

/// Read an optional environment variable, returning `Option<String>`.
///
/// # Examples
///
/// ```rust,ignore
/// let redis = vega::env_opt!("REDIS_URL"); // Option<String>
/// ```
#[macro_export]
macro_rules! env_opt {
    ($name:literal) => {{
        std::env::var($name).ok()
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn env_optional_macro() {
        let value = crate::env_opt!("__VEGA_TEST_NOT_SET__");
        assert!(value.is_none());
    }
}
