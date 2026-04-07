# Vega (वेग) — Flow Diagrams
> Visual reference for the pure Axum routing flows inside the framework.

---

## Diagram 1: Full Build Pipeline

```
┌─────────────────────────────────────────────────────────┐
│                    Developer Action                      │
│              Creates pages/blog/[slug].rs                │
└──────────────────────────┬──────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                  cargo build triggered                   │
└──────────────────────────┬──────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                    build.rs runs                         │
│                                                          │
│  ┌──────────────┐    ┌──────────────┐                   │
│  │  Scan pages/ │    │   Scan api/  │                   │
│  └──────┬───────┘    └──────┬───────┘                   │
│         │                   │                            │
│         ▼                   ▼                            │
│  ┌──────────────────────────────────┐                   │
│  │         vega-router Crate        │                   │
│  │  Parses filenames + AST Nodes    │                   │
│  └──────────────────┬───────────────┘                   │
│                     │                                    │
│         ┌───────────┼───────────┐                       │
│         ▼           ▼           ▼                       │
│  runtime_app.rs  runtime_pages.rs  runtime_api.rs       │
│  (State binds)  (Routes setup)    (Axum routes)         │
│                                                          │
│  Written to: target/debug/build/.../out/                │
└──────────────────────────┬──────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│              src/ compilation begins                     │
│                                                          │
│  main.rs                                                 │
│    include!(concat!(env!("OUT_DIR"), "main_gen.rs"));    │
│                                                          │
│  Compiler resolves all dynamic route definitions         │
└──────────────────────────┬──────────────────────────────┘
                           │
                           ▼
┌─────────────────────┐    │
│   Server Binary     │    │
│  (Native Target)    │◄───┘
│                     │
│  - Axum server      │
│  - Static assets    │
│  - Semantic HTML    │
└─────────────────────┘
```

---

## Diagram 2: HTTP Request Lifecycle (Axum Only)

```
Browser                    Vega Server (Axum)                   Database
   │                              │                                 │
   │  GET /blog/hello-world       │                                 │
   │─────────────────────────────►│                                 │
   │                              │                                 │
   │                    ┌─────────┴──────────┐                     │
   │                    │  Global Middleware  │                     │
   │                    │  Logger, CORS, etc  │                     │
   │                    └─────────┬──────────┘                     │
   │                              │                                 │
   │                    ┌─────────┴──────────┐                     │
   │                    │  Route Middleware   │                     │
   │                    │  (e.g., auth checks)│                     │
   │                    └─────────┬──────────┘                     │
   │                              │                                 │
   │                    ┌─────────┴──────────┐                     │
   │                    │  handler matching   │                     │
   │                    │                     │                     │
   │                    │  Extractors:        │                     │
   │                    │  - State<AppState>  │                     │
   │                    │  - Path(slug)       │                     │
   │                    └─────────┬──────────┘                     │
   │                              │  SELECT * FROM posts             │
   │                              │  WHERE slug = 'hello-world'      │
   │                              │─────────────────────────────────►│
   │                              │                                   │
   │                              │◄─────────────────────────────────│
   │                              │  Post { title, content, ... }    │
   │                    ┌─────────┴──────────┐                     │
   │                    │  String Formatting  │                     │
   │                    │  (Template engine / │                     │
   │                    │  Raw HTML payload)  │                     │
   │                    └─────────┬──────────┘                     │
   │                              │                                 │
   │  200 OK                      │                                 │
   │  Content-Type: text/html     │                                 │
   │  (Semantic HTML response)    │                                 │
   │◄─────────────────────────────│                                 │
   │                                                                 │
   │  [Browser renders semantic HTML directly]                       │
   │  [HTMX handles DOM swapping if requested via layout targets]    │
```

---

## Diagram 3: HTMX Interaction Flow (Post Mutator)

```
Browser (HTMX)             Vega Server                    API Server / DB
   │                              │                            │
   │  User Submits Form           │                            │
   │  POST /api/users/123         │                            │
   │  HX-Request: true            │                            │
   │─────────────────────────────►│                            │
   │                              │                            │
   │                    ┌─────────┴──────────┐                │
   │                    │  post_handler()     │                │
   │                    │  (resolved mapped   │                │
   │                    │  by build.rs from   │                │
   │                    │  api/users/[id].rs) │                │
   │                    └─────────┬──────────┘                │
   │                              │  INSERT INTO users...       │
   │                              │───────────────────────────►│
   │                              │                            │
   │  200 OK                      │◄───────────────────────────│
   │  Content-Type: text/html     │                            │
   │  <div id="users-list">...    │                            │
   │◄─────────────────────────────│                            │
   │                                                           │
   │  [HTMX swaps <div id="users-list"> with response]         │
```

---

## Diagram 4: File System → Routes Setup

```
FILE SYSTEM                   ROUTE                    AXUM BINDING IN main_gen.rs
─────────────────────────────────────────────────────────────────────────
pages/
├── index.rs              →   /                   →   app.route("/", get(crate::pages::index::handler))
├── about.rs              →   /about              →   app.route("/about", get(crate::pages::about::handler))
│
├── blog/
│   ├── index.rs          →   /blog               →   app.route("/blog", get(crate::pages::blog::index::handler))
│   └── [slug].rs         →   /blog/:slug         →   app.route("/blog/:slug", get(crate::pages::blog::slug::handler))
│
└── (auth)/
    └── login.rs          →   /login              →   app.route("/login", get(crate::pages::login::handler))
                                                      .route("/login", post(crate::pages::login::post_handler))
```

---

## Diagram 5: Middleware Execution Chain

```
Incoming Request
       │
       ▼
┌──────────────────────────────────────────────┐
│              TOWER LAYER STACK               │
│  (assembled by Axum automatically)           │
│                                              │
│  1. TraceLayer        ← always on            │
│     Logs method, path, status, latency       │
│                                              │
│  2. RequestIdLayer    ← always on            │
│     Adds X-Request-ID header                 │
│                                              │
│  3. CompressionLayer  ← if configured        │
│     Brotli/gzip response compression         │
└────────────────────────┬─────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────┐
│  Route-specific Middleware                   │
│                                              │
│  Defined dynamically in pages via macro:     │
│  #[vega::page(middleware=["my_guard"])]      │
│                                              │
│  Resolves to:                                │
│  axum::middleware::from_fn(my_guard)         │
└────────────────────────┬─────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────┐
│  Final Extractor Binding                     │
│  (State injection, Parameters extraction)    │
│  handler() executed                          │
└──────────────────────────────────────────────┘
```
