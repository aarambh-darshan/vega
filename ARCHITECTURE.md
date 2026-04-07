# Vega (वेग) — Architecture
> A Next.js-inspired full-stack framework for Rust, built on pure Axum.
> File-based routing · Zero-config · Co-located API routes

---

## 1. Philosophy

Vega (वेग — Sanskrit for *speed, momentum*) exists because Axum is powerful but requires manual wiring.
Next.js succeeded not because React was bad, but because it removed friction.
Vega does the same for Axum:

- **Convention over configuration** — drop a file in `pages/`, get a route
- **Pure Server Rendering** — return standard HTML payloads, perfectly mirroring paradigms like HTMX
- **Co-located API routes** — your backend endpoint lives where it belongs
- **One `cargo run`** — starts the entire server seamlessly
- **Automated Wiring** — no `pub mod` exports needed; everything is auto-linked via `build.rs` codegen

Vega is NOT a replacement for Axum. It is a **structured layer on top of it**. Your models, database connections, and business logic remain pure Rust.

---

## 2. Rendering & Response Model

Every page in Vega handles requests and returns pure HTML strings or API responses.

```rust
// pages/index.rs
use vega::server::web::{Html, Request};

#[vega::page]
pub fn IndexPage() -> &'static str {
    "Hello from Vega"
}

// Every page exports a required `handler`
pub async fn handler(_req: Request) -> Html<String> {
    Html(format!("<h1>Welcome to Vega</h1>"))
}
```

Dynamic logic, database queries, and session management are executed directly within the standard `handler` function. No WASM hydration boundary constraints exist—shipping purely fast, server-rendered views.

---

## 3. Project Structure

```
my-vega-app/
│
├── Cargo.toml                       # standard Rust workspace
│
├── app.rs                           # (Optional) Application State and initialization
├── middleware.rs                    # (Optional) Global or scope-specific Tower middleware
│
├── pages/                           # FILE-BASED ROUTING ROOT
│   ├── index.rs                     # → GET /
│   ├── about.rs                     # → GET /about
│   │
│   ├── blog/
│   │   ├── index.rs                 # → GET /blog
│   │   └── [slug].rs                # → GET /blog/:slug  (dynamic segment)
│   │
│   ├── docs/
│   │   └── [...path].rs             # → GET /docs/*  (catch-all segment)
│   │
│   └── (auth)/                      # route group — no URL segment added
│       ├── login.rs                 # → GET /login
│       └── register.rs              # → GET /register
│
├── api/                             # CO-LOCATED API ROUTES
│   ├── hello.rs                     # → GET/POST /api/hello
│   ├── auth/
│   │   ├── login.rs                 # → POST /api/auth/login
│   │   └── logout.rs                # → POST /api/auth/logout
│   └── blog/
│       └── [slug].rs                # → GET /api/blog/:slug
│
├── public/                          # static assets served at /
│   ├── favicon.ico
│   └── images/
│
├── src/                             # Auto-generated entry point inclusion
│   └── main.rs
│
└── target/                          # Rust build output
    └── build/                       # Vega's internal codegen output (router.rs)
```

---

## 4. File-Based Routing — Rules

### 4.1 Static Routes
```
pages/index.rs         →  /
pages/about.rs         →  /about
pages/blog/index.rs    →  /blog
pages/blog/hello.rs    →  /blog/hello
```

### 4.2 Dynamic Segments
```
pages/blog/[slug].rs          →  /blog/:slug
pages/shop/[id]/reviews.rs    →  /shop/:id/reviews
```

Inside the file, you can parse dynamic parameters cleanly via Axum's `Path` extractor inside the `handler`.

### 4.3 Catch-All Segments
```
pages/docs/[...path].rs    →  /docs/*  (matches /docs/a/b/c)
```

### 4.4 Route Groups (no URL segment)
```
pages/(auth)/login.rs      →  /login   (auth group, no /auth/ prefix)
```

### 4.5 Auto-POST Detection
Vega's compiler parses your files for explicitly named functions to automate API verb registration:
- `pub async fn handler` → GET endpoint
- `pub async fn post_handler` → POST endpoint (e.g., standard form submissions or HTMX mutations)

---

## 5. Middleware & State

### 5.1 Application State
If you place an `app.rs` file at your project root, Vega discovers it automatically. You just define:
```rust
pub struct AppState { ... }
pub async fn create_state() -> AppState { ... }
```
Vega automatically embeds this into Axum's `.with_state(state)` across all generated routers.

### 5.2 Middleware
Defining global or scoped middleware is as simple as creating `middleware.rs` in the project root:
```rust
use axum::{middleware::Next, response::Response, extract::Request};

// Discovered dynamically by Vega and available for page attribution
pub async fn auth_guard(req: Request, next: Next) -> Response {
    next.run(req).await
}
```

Apply to pages explicitly:
```rust
// pages/dashboard.rs
#[vega::page(middleware = ["crate::middleware::auth_guard"])]
pub fn Dashboard() -> &'static str { "Dashboard" }

pub async fn handler(...) -> Html<String> { ... }
```

---

## 6. Vega Codegen — How It Works

Vega uses a **build script (`build.rs`)** that:

1. **Scans** `pages/`, `api/`, and root directories at compile time.
2. **Parses** Rust syntax using `syn` to extract state definitions, middleware identifiers, and POST handlers.
3. **Generates** runtime artifacts inside `/target/debug/build/.../out/runtime_api.rs` (or similar output paths).
4. **Includes** these generated routes seamlessly inside the minimalist `src/main.rs`.

This means **you never touch routing or application entrypoint code manually**. You just add feature files. Axum scales effortlessly across these automatically bound endpoints.
