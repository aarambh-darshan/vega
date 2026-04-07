# Vega — Project Structure & Module System
> A simple file-based architecture auto-wiring purely through Axum.

---

## 1. The Zero-Config Concept Workspace
Vega projects don't require manual `pub mod module_name` registration inside a central `src/main.rs`. By using a Rust `build.rs` compile-time hook, the framework recursively identifies files in routing directories (`pages/` and `api/`), along with essential domain configuration files (`app.rs` and `middleware.rs`).

It then automatically wires up the corresponding `Axum` routers to point to the handlers within those files!

---

## 2. Typical Project Tree

```
my-vega-app/
│
├── build.rs                   ← Vega's codegen entry point
│
├── Cargo.toml                 ← Standard workspace root
│   [dependencies]
│   vega = "0.9"
│   axum = "0.8"
│   tokio = { version = "1", features = ["full"] }
│
├── app.rs                     ← Defines global `AppState` and initialization
├── middleware.rs              ← Contains reusable Tower middleware handlers
│
├── pages/                     ← FILE-BASED FRONTEND ROUTING ROOT
│   │
│   ├── index.rs               → GET /
│   │
│   ├── about.rs               → GET /about
│   │
│   ├── _not_found.rs          → Handles 404
│   │
│   ├── blog/
│   │   ├── index.rs           → GET /blog
│   │   └── [slug].rs          → GET /blog/:slug (dynamic segment matching handler arguments)
│   │
│   ├── docs/
│   │   └── [...path].rs       → GET /docs/* (catch-all)
│   │
│   └── (auth)/                → Route group (no URL prefix added)
│       ├── login.rs           → GET /login (and auto-POST if `post_handler` is defined)
│       └── register.rs        → GET /register
│
├── api/                       ← CO-LOCATED BACKEND HANDLERS
│   │                            Same file convention as pages/
│   │
│   ├── hello.rs               → GET /api/hello
│   │
│   └── users/
│       ├── index.rs           → GET /api/users
│       └── [id].rs            → GET /api/users/:id
│
├── public/                    ← STATIC ASSETS (served at /)
│   ├── favicon.ico
│   └── images/
│
├── styles/
│   └── global.css
│
└── src/                       ← BOOTSTRAPPED ENTRY POINT
    └── main.rs                ← Single-line `include!` generated code
```

---

## 3. The `build.rs` Scanner Details

Vega generates a pure Rust payload directly attached to `main.rs` via `include_str`. You do not write router logic at all.
The `vega-router` crate scans directories recursively and outputs generated files directly to Cargo's `OUT_DIR`.

These generated files bundle up the definitions:
1. `runtime_app.rs` — Wires `app.rs` logic into an `.with_state(state)` builder.
2. `runtime_pages.rs` — Attaches all URLs mapped from the directory hierarchy via `axum::routing::get(handler)`.
3. `runtime_api.rs` — Attaches pure JSON-driven handler logic under `/api/...`.
4. `main_gen.rs` — Serves as the overarching Tokio executor initialization.

### Dynamic Segment Naming
For files containing non-alphanumeric identifiers (like `[slug].rs`), the codegen module maps the system file name to a sanitized valid Rust namespace token but retains the original naming parameter for Axum's matching system cleanly.

---

## 4. The Core Components

### `app.rs`
Create an application state struct (typically bundling a database pool, configurations, session states):
```rust
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
}

pub async fn create_state() -> AppState {
    AppState { db: init_db().await }
}
```

### `pages/*.rs`
Export an Axum-compliant handler:
```rust
use axum::extract::{State, Path};
use vega::server::web::{Html, Request};
use crate::app::AppState;

#[vega::page]
pub fn IndexPage() -> &'static str { "Dynamic Blog" }

pub async fn handler(
    State(state): State<AppState>,
    Path(slug): Path<String>
) -> Html<String> {
    Html(format!("Post: {}", slug))
}
```

### `middleware.rs`
Define reusable guards to apply iteratively using the macro annotations (`#[vega::page]`).
```rust
use axum::{middleware::Next, response::Response, extract::Request};

pub async fn require_login(req: Request, next: Next) -> Response {
    // Session token check
    next.run(req).await
}
```

---

## Summary: Focus on Domain State over Syntax

Through the compile-time hooks, you achieve near-instant routing. Simply creating a file named `pages/shop/[id].rs` immediately guarantees the application will listen on `/shop/:id` returning whatever semantic payload you inject without worrying about mod trees or router chains!
