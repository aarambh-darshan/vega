# Vega — Under the Hood
> Deep internals: how the pure Axum routing system actually works

---

## 1. The Build Pipeline

Vega's magic happens entirely at **compile time**. There is no runtime file system scanner, no messy reflections, no dynamic macro loading. Everything is resolved before the binary even hits `cargo build` through pure AST code generation mechanisms.

```
Developer writes code
        ↓
cargo build triggers
        ↓
build.rs runs FIRST (before src/ is compiled)
  ├── Search app.rs and middleware.rs for system configurations
  ├── Scan pages/   → generating runtime_pages.rs route attachments
  ├── Scan api/     → generating runtime_api.rs route attachments
  └── Generate main_gen.rs encompassing root Tokio configuration
        ↓
src/ compiles
  ├── main.rs automatically includes `main_gen.rs`
  ├── Compiler resolves routes to valid modules using standard Rust imports
  └── Zero overhead — generated code scales natively with pure Axum performance
        ↓
Binary produced
```

**Why compile-time?** 
Because Rust's type system allows us to rigorously guarantee routing without sacrificing performance. There is no string interpolation routing layer holding up requests at runtime. The file tree *is* the AST.

---

## 2. The vega-router Crate — Internal Magic

`vega-router` is the source-of-truth compiler running inside `build.rs`.

### AST Node Resolution

Unlike standard scanners that guess at file contexts, `vega-router` natively utilizes `syn` to parse Rust structures inside your files, ensuring the auto-generated routing tree perfectly maps to the exposed method signatures.

```rust
// Simplified internal matching mechanism
pub fn introspect_file(path: &Path) -> RouteCapabilities {
    let source = fs::read_to_string(path);
    let syntax = syn::parse_file(&source)?;

    let mut has_get = false;
    let mut has_post = false;
    let mut middlewares = Vec::new();

    // Iterate through attributes inside the `#[vega::page]`
    for item in syntax.items {
        // ... Determine whether this file exports `handler` or `post_handler`
        // ... Map any configured middleware string arguments
    }
    
    RouteCapabilities { has_get, has_post, middlewares }
}
```

### URL Slug Processing

Filenames like `[slug].rs` are intuitively converted to Axum named parameter path strings (`/:slug`) internally:

```rust
fn path_to_url_pattern(filepath: &str) -> String {
    filepath
        .replace("[...", "*")       // [...path] → /*path (catchall) 
        .replace("[", ":")          // [id] → :id
        .replace("]", "")
        .replace("(auth)/", "")     // Routing groups stripped physically
}
```

---

## 3. The #[vega::page] Macro

Currently, `#[vega::page]` operates primarily as an analytical marker, allowing developers to configure route-level properties simply cleanly:

```rust
#[vega::page(middleware = ["my_logger"])]
pub fn AccountPage() -> &'static str { "Account Options" }
```

When evaluated, `vega-router` picks up the associated metadata to inject the correct `axum::middleware::from_fn(...)` bindings around the associated path dynamically.

---

## 4. State Management Injection

Axum requires complex type-matching for State references dynamically scaling across deep configurations. Vega handles this transparently.

If `app.rs` contains `AppState`:
1. `build.rs` creates a wrapper configuration invoking `crate::app::create_state()`.
2. Appends `.with_state(state)` across isolated sub-routers transparently.

```rust
// Auto-generated runtime_api.rs representation:
pub fn register_api(mut app: Router<()>, state: AppState) -> Router<()> {
    app = app.route("/api/hello", get(crate::api::hello::handler));
    
    // Transparently bind the state resolving dependency chains across scopes
    app.with_state(state)
}
```

If no `app.rs` is present, it elegantly falls back to using standard anonymous states (`()`) preventing complex boilerplate requirements.

---

## 5. HTMX / Mutation Support Built-in

Because routes map one-to-one to Axum routing bindings, returning specific component fragments handles standard HTMX mechanics natively:

```rust
// pages/users/[id].rs

// Discovered dynamically: mapped as `get(handler)`
pub async fn handler(Path(id): Path<u32>) -> Html<String> {
    Html(format!("<form hx-post='/users/{}'>...</form>", id))
}

// Discovered dynamically: mapped as `post(post_handler)`
pub async fn post_handler(Path(id): Path<u32>) -> Html<String> {
    update_user(id).await;
    Html("<div id='status'>User updated successfully!</div>".into())
}
```

No additional configuration logic is needed. The `codegen` logic understands these explicit function keys securely mapping them safely to backend REST verbs directly into Axum.
