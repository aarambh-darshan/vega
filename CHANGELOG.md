# Changelog

All notable changes to the Vega framework are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.0] — 2026-04-07

### Added

- **`vega-server`**: `init_tracing()` — initialize tracing subscriber with `RUST_LOG` support
- **`vega-server`**: `run_with_shutdown()` — graceful shutdown on Ctrl+C (SIGINT)
- **`vega-server`**: `static_file_service()` — serve static files from `public/` directory
- **`vega-server`**: `security_headers_layer()` — X-Content-Type-Options, X-Frame-Options, X-XSS-Protection
- **`vega-server`**: `ApiResponse::created()` — 201 Created response helper
- **`vega-server`**: `ApiResponse::no_content()` — 204 No Content response helper
- **`vega-server`**: `ApiError::not_found()` — 404 error helper
- **`vega-server`**: `ApiError::conflict()` — 409 error helper
- **`vega-server`**: `ApiError::unprocessable()` — 422 error helper
- **`vega-server`**: `make_session_cookie()` / `clear_session_cookie()` — cookie lifecycle helpers
- **`vega-server`**: `esc()` — HTML escape convenience function
- **`vega-server`**: `PageContext` struct for SSR page context
- **`vega-server`**: `delete` and `patch` added to `web` re-exports
- **`vega-server`**: `ApiResponse::redirect()` now includes `Location` header
- **`vega-core`**: `html_escape()` utility function
- **`vega-core`**: `Display` implementations for `RenderMode`, `HttpMethod`, `SegmentKind`
- **`vega-core`**: `RenderMode::is_server_rendered()` / `is_client_only()` predicates
- **`vega-core`**: `SegmentKind::is_special_file()` / `has_url_segment()` predicates
- **`vega-core`**: `RouteManifest::total_routes()` method
- **`vega-client`**: `serialize_hydration_data()` function
- **`vega`**: Re-exports `ApiError`, `ApiRequest`, `ApiResponse` at top level
- **`vega`**: `security_headers_layer` in middleware module
- **`vega`**: `data` module re-exporting all fetch utilities
- **Project**: MIT OR Apache-2.0 dual license
- **Project**: CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md
- **Project**: GitHub Actions CI (check, test, clippy, fmt, doc)
- **Project**: rustfmt.toml configuration
- **Demo**: Full admin panel with create/delete CRUD operations
- **Demo**: Modern CSS design system (Inter font, cards, grids, responsive, dark accents)
- **Demo**: 6 seed blog posts with rich content
- **Demo**: Search, tag filtering, and pagination for blog
- **Demo**: Static file serving from `public/` directory
- **Demo**: Tracing initialization with request logging
- **Demo**: Graceful shutdown support
- **Demo**: Security headers middleware
- **Demo**: Flash message support in layout
- **Demo**: Improved error pages (404, 403)
- **Demo**: Documentation page with code examples
- **Demo**: About page with architecture reference

### Changed

- **Version**: Bumped from 0.8.0 to 0.9.0
- **License**: Changed from UNLICENSED to MIT OR Apache-2.0
- **`vega-server`**: 10MB body size limit for `ApiRequest::from_axum_request()`
- **`vega-core`**: Added `Default` derive for `RenderMode` (defaults to `Ssr`)
- **`vega-core`**: Added `Default` derive for `RouteManifest`
- **All crates**: Added comprehensive `///` doc comments on all public items
- **All crates**: Added crate-level `//!` documentation
- **All crates**: Added `description` field to Cargo.toml
- **All crates**: Added `authors`, `repository`, `license` workspace fields
- **README**: Complete rewrite with usage examples, architecture, CLI reference
- **CHANGELOG**: Rewritten in Keep a Changelog format

### Fixed

- **`vega-server`**: `ApiResponse::redirect()` now returns HTTP 307 with `Location` header instead of just JSON
- **Demo**: Admin page properly handles request body consumption after reading extensions
- **Demo**: All pages use consistent 4-argument `render_layout()` with flash message support
- **Demo**: HTML escaping uses `vega::core::html_escape()` consistently

## [0.8.0] — 2026-04-06

### Added

- Full blog + auth demo application (`examples/blog-auth-demo`)
- Session-based authentication with `InMemoryAuthService` and `InMemorySessionStore`
- Role-based middleware guards (`require_auth`, `require_admin`)
- Blog with search, tag filtering, and sorting
- API routes for auth (login, register, me) and posts (CRUD)
- Dynamic route params (`[slug].rs`) and catch-all (`[...path].rs`)
- Route groups (`(auth)/`) for shared layout without URL segments
- Components and sections system for page composition
- `ApiRequest` with typed JSON, form, query parsing
- `ApiResponse` with status helpers and cookie support
- `parse_cookie_map()` for session cookie extraction

## [0.7.0] — 2026-04-05

### Added

- `vega-fetch` crate: `use_query`, `use_mutation`, `use_params`, `use_search_params`
- `SearchParamHandle` for reactive URL search parameter manipulation
- `#[server_fn]` macro for RPC-style server functions
- `FetchError` enum with typed error variants

## [0.6.0] — 2026-04-04

### Added

- `vega-client` crate with `vega_hydrate!` macro
- `parse_hydration_data()` for client-side state restoration
- SSR/CSR feature flag split in `vega-core` and `vega-fetch`

## [0.5.0] — 2026-04-03

### Added

- `vega-server` crate with Axum integration
- `run()` function for starting the HTTP server
- `TraceLayer`, `CorsLayer`, `CompressionLayer`, `RateLimitLayer` middleware
- `ApiResponse::json()` and `ApiResponse::status_json()`
- `ApiError` with `bad_request`, `unauthorized`, `forbidden`, `internal`
- Request ID generation and propagation
- Health check endpoint (`/health`)
- Route introspection endpoint (`/api/_vega/routes`)

## [0.4.0] — 2026-04-02

### Added

- `vega-macros` proc macro crate
- `#[page]`, `#[layout]`, `#[get]`, `#[post]`, `#[put]`, `#[delete]`, `#[patch]` proc macros
- `#[server_fn]` macro with optional `cache` attribute
- `PageMeta` and `ApiMeta` constant generation

## [0.3.0] — 2026-04-01

### Added

- `vega-router` crate: file-based route scanning and Rust code generation
- `scan_pages()`, `scan_api()`, `scan_generic()` directory scanners
- `generate_all()` entry point for `build.rs`
- Segment parsing: static, dynamic `[param]`, catch-all `[...param]`, groups `(name)`
- Special file detection: `_layout`, `_error`, `_loading`, `_not_found`, `index`
- Axum-compatible route path generation (`:param` → `{param}`)

## [0.2.0] — 2026-03-31

### Added

- `vega-config` crate: `Vega.toml` configuration parser
- `VegaConfig` with `app`, `server`, `build`, `features`, `ssr`, `ssg`, `auth` sections
- `from_path()` and `from_str()` constructors

## [0.1.0] — 2026-03-30

### Added

- `vega-core` crate: core types and enums
- `RenderMode` (SSR, SSG, CSR, ISR)
- `HttpMethod` (GET, POST, PUT, PATCH, DELETE)
- `SegmentKind` (Static, Index, Dynamic, CatchAll, Group, Layout, Error, Loading, NotFound)
- `PageMeta`, `ApiMeta`, `RouteEntry`, `ApiRouteEntry`, `RouteManifest`
- `vega-cli` crate with `new`, `dev`, `build`, `generate`, `routes` commands
- `vega` facade crate with module re-exports and `env!`/`env_opt!` macros
