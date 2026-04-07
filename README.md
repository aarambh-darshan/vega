# वेग Vega

> [!WARNING]
> **IMPORTANT NOTICE:** This framework is built strictly for **reading and learning purposes**. It is an experimental project and **should NOT be used in production environments**.

**A Next.js-inspired full-stack web framework for Rust.**

Vega (वेग — Sanskrit for *speed, momentum*) provides file-based routing, server-side rendering, co-located API routes, typed parameters, session auth, and zero-config conventions — all compiled into a single binary.

[![CI](https://github.com/AarambhDevHub/vega/actions/workflows/ci.yml/badge.svg)](https://github.com/AarambhDevHub/vega/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

---

## Features

- **📁 File-Based Routing** — Drop a `.rs` file in `pages/`, get a route. Dynamic `[slug].rs`, catch-all `[...path].rs`, route groups `(auth)/`.
- **⚡ Multiple Render Modes** — SSR and SSG rendered payloads via `#[page(mode = "ssr")]`.
- **🔌 Co-Located API Routes** — Files in `api/` become REST endpoints with `#[get]`, `#[post]`, `#[put]`, `#[delete]`.
- **🧩 Nested Layouts** — `_layout.rs` wraps child routes. Special files: `_error.rs`, `_loading.rs`, `_not_found.rs`.
- **🔒 Auth & Sessions** — Built-in session cookies with HttpOnly, SameSite. Role-based middleware guards.
- **🔍 Typed Parameters** — `use_params`, `use_search_params`, typed query deserialization.
- **🛠 CLI Tooling** — `vega new`, `vega dev`, `vega build`, `vega generate`, `vega routes`.
- **🏗 Build-Time Codegen** — Routes scanned and compiled at `cargo build`. No runtime reflection.
- **📦 Single Binary** — Deploy one executable. No Node.js, no runtime dependencies.
- **🛡 Production Middleware** — CORS, compression, rate limiting, security headers, request tracing.

## Quick Start

### Create a New Project

```bash
# Install the CLI
cargo install vega-cli

# Create a new Vega project
vega new my-app
cd my-app
vega dev
```

### Or Add to an Existing Project

```toml
# Cargo.toml
[dependencies]
vega = { git = "https://github.com/aarambh-darshan/vega.git", branch = "main", features = ["ssr"] }

[build-dependencies]
vega-router = { git = "https://github.com/aarambh-darshan/vega.git", branch = "main" }
```

### Project Structure

```
my-app/
├── pages/                  # File-based page routes
│   ├── index.rs            # → GET /
│   ├── about.rs            # → GET /about
│   ├── _layout.rs          # Wraps all child routes
│   ├── _not_found.rs       # Custom 404 page
│   ├── blog/
│   │   ├── index.rs        # → GET /blog
│   │   ├── [slug].rs       # → GET /blog/:slug
│   │   └── _layout.rs      # Blog-specific layout
│   ├── docs/
│   │   └── [...path].rs    # → GET /docs/*path (catch-all)
│   └── (auth)/             # Route group (no URL segment)
│       ├── login.rs        # → GET /login
│       └── register.rs     # → GET /register
├── api/                    # Co-located API routes
│   ├── auth/
│   │   ├── login.rs        # → POST /api/auth/login
│   │   └── register.rs     # → POST /api/auth/register
│   └── posts/
│       ├── index.rs        # → GET|POST /api/posts
│       └── [id].rs         # → GET|PUT|DELETE /api/posts/:id
├── components/             # Shared UI components
├── sections/               # Landing page sections
├── src/
│   ├── main.rs             # Entry point
│   ├── runtime.rs          # Server setup + middleware
│   └── state.rs            # App state
├── public/                 # Static files (served at /)
├── Vega.toml               # Framework configuration
├── Cargo.toml
└── build.rs                # Codegen entry point
```

## Page Routes

Every `.rs` file in `pages/` automatically becomes a route:

```rust
// pages/index.rs → GET /
#[vega::page(mode = "ssr")]
pub fn IndexPage() -> &'static str {
    "Home page"
}

pub async fn handler(
    State(state): State<AppState>,
    req: Request,
) -> Html<String> {
    Html(render_layout("Home", &render(), None, None))
}
```

### Render Modes

| Mode | Attribute | Behavior |
|------|-----------|----------|
| SSR | `mode = "ssr"` | Rendered dynamically on every request (default) |
| SSG | `mode = "ssg"` | Rendered at build time |

### Dynamic Routes

```rust
// pages/blog/[slug].rs → GET /blog/:slug
pub async fn handler(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Html<String> {
    // slug = "hello-world" for /blog/hello-world
}

// pages/docs/[...path].rs → GET /docs/*path
pub async fn handler(
    Path(path): Path<String>,
) -> Html<String> {
    // path = "api/auth/session" for /docs/api/auth/session
}
```

### Middleware

```rust
// pages/dashboard.rs
#[vega::page(mode = "csr", middleware = [auth::require_auth])]
pub fn DashboardPage() -> &'static str {
    "Protected page"
}
```

## API Routes

Files in `api/` become JSON API endpoints:

```rust
// api/posts/index.rs
#[vega::get]
pub fn ListPosts() -> &'static str { "GET /api/posts" }

#[vega::post(middleware = [auth::require_auth])]
pub fn CreatePost() -> &'static str { "POST /api/posts" }

pub async fn get_handler(State(state): State<AppState>) -> impl IntoResponse {
    let posts = state.posts.lock().unwrap().clone();
    ApiResponse::json(posts)
}

pub async fn post_handler(
    State(state): State<AppState>,
    Json(body): Json<CreatePostBody>,
) -> impl IntoResponse {
    // create post...
    ApiResponse::created(json!({"id": new_id}))
}
```

## Configuration

```toml
# Vega.toml
[app]
name = "my-app"
base_url = "http://localhost:3000"

[server]
host = "127.0.0.1"
port = 3000

[build]
out_dir = "dist"
public_dir = "public"

[features]
tailwind = false
compress = false
source_maps = true

[ssr]
streaming = true

[auth]
provider = "in-memory"
```

## Architecture

Vega is split into focused crates:

| Crate | Purpose |
|-------|---------|
| `vega` | Facade — re-exports everything |
| `vega-core` | Core types, enums, data structures |
| `vega-config` | `Vega.toml` configuration parser |
| `vega-router` | File-based route scanner + compile-time codegen |
| `vega-macros` | Proc macros (`#[page]`, `#[get]`, `#[layout]`, etc.) |
| `vega-server` | Axum runtime, API handling, auth, middleware |
| `vega-fetch` | Data fetching, typed params, search params |
| `vega-cli` | CLI binary (`vega new`, `dev`, `build`, `routes`) |

### Build Pipeline

```
pages/*.rs + api/*.rs
       │
       ▼
vega-router (build.rs)
  ├── Scan files → RouteEntry[]
  ├── Parse segments → SegmentKind
  └── Emit Rust code:
       ├── pages_mod.rs    (module declarations)
       ├── api_mod.rs
       ├── router.rs       (page route table)
       ├── api_router.rs   (API route table)
       ├── runtime_pages.rs (Axum route registration)
       └── runtime_api.rs
              │
              ▼
       main.rs (include! generated code)
              │
              ▼
       Single compiled binary
```

## CLI Commands

```bash
# Create a new project
vega new my-app

# Start development server
vega dev

# Build for production (SSG pages)
vega build

# Generate route files
vega generate page about
vega generate api users

# List all routes
vega routes
```

## Running the Demo

```bash
git clone https://github.com/AarambhDevHub/vega.git
cd vega
cargo run -p blog-auth-demo

# Visit http://localhost:3000
# Demo accounts:
#   admin@vega.dev / admin123
#   member@vega.dev / member123
```

The demo showcases:
- File-based routing with all segment types
- Blog with search, tag filtering, and pagination
- Session auth with login/register/logout
- Admin panel with CRUD operations
- Role-based middleware guards
- API endpoints with JSON responses
- Dynamic `[slug]` and catch-all `[...path]` routes
- Nested layouts and route groups

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
