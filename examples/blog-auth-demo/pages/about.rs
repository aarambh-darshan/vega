use crate::app::{current_user_from_headers, AppState};
use vega::server::web::{Html, Request, State};

#[vega::page(mode = "ssg")]
pub fn AboutPage() -> &'static str {
    "About"
}

pub fn render() -> String {
    r#"<h1>About Vega</h1>

    <p>Vega (वेग — Sanskrit for <em>speed, momentum</em>) is a Next.js-inspired full-stack web framework for Rust.</p>

    <div class="card" style="margin-top:1.5rem">
        <h3>Why Vega?</h3>
        <p>Rust gives you unmatched performance and safety, but building web apps requires a lot of boilerplate. Vega provides the conventions and tooling to go from idea to production fast:</p>
        <ul style="color:var(--ink-muted);margin-top:0.5rem;padding-left:1.25rem">
            <li>File-based routing with zero config</li>
            <li>Server-side rendering out of the box</li>
            <li>Co-located API routes with typed JSON/form parsing</li>
            <li>Session auth with role-based middleware guards</li>
            <li>Build-time code generation — no runtime reflection</li>
            <li>Single binary deployment</li>
        </ul>
    </div>

    <div class="card">
        <h3>Architecture</h3>
        <p>Vega is split into focused crates:</p>
        <table>
            <thead><tr><th>Crate</th><th>Purpose</th></tr></thead>
            <tbody>
                <tr><td><code>vega-core</code></td><td>Core types, enums, and data structures</td></tr>
                <tr><td><code>vega-config</code></td><td>Vega.toml configuration parsing</td></tr>
                <tr><td><code>vega-router</code></td><td>File-based route scanner and compile-time codegen</td></tr>
                <tr><td><code>vega-macros</code></td><td>Procedural macros (#[page], #[get], #[post], etc.)</td></tr>
                <tr><td><code>vega-server</code></td><td>Axum integration, SSR, middleware, auth</td></tr>
                <tr><td><code>vega-client</code></td><td>Client-side hydration (planned)</td></tr>
                <tr><td><code>vega-fetch</code></td><td>Data fetching, typed params, search params</td></tr>
                <tr><td><code>vega-cli</code></td><td>CLI binary (vega new, dev, build, routes)</td></tr>
                <tr><td><code>vega</code></td><td>Facade crate — re-exports everything</td></tr>
            </tbody>
        </table>
    </div>

    <div class="card">
        <h3>This Demo</h3>
        <p>This blog + auth application demonstrates:</p>
        <ul style="color:var(--ink-muted);margin-top:0.5rem;padding-left:1.25rem">
            <li>File-based pages: <code>pages/index.rs</code>, <code>pages/blog/[slug].rs</code></li>
            <li>API routes: <code>api/auth/login.rs</code>, <code>api/posts/index.rs</code></li>
            <li>Route groups: <code>pages/(auth)/login.rs</code></li>
            <li>Dynamic params: <code>[slug]</code> and catch-all <code>[...path]</code></li>
            <li>Middleware: auth guards, admin guards, logging</li>
            <li>Session cookies with HttpOnly, SameSite</li>
            <li>Admin CRUD operations</li>
        </ul>
    </div>"#
        .to_string()
}

pub async fn handler(State(state): State<AppState>, req: Request) -> Html<String> {
    let user = current_user_from_headers(req.headers(), &state).await;
    let body = render();
    Html(crate::pages::_layout::render_layout(
        "About",
        &body,
        user.as_ref(),
        None,
    ))
}
