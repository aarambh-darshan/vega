use crate::app::{current_user_from_headers, AppState};
use vega::server::web::{Html, Request, State};

#[vega::page(mode = "ssg")]
pub fn IndexPage() -> &'static str {
    "Welcome to the Vega Blog + Auth demo"
}

pub fn render() -> String {
    format!(
        r#"{hero}
        <section class="section">
            <div class="section-title">
                <h2>Framework Features</h2>
                <p>Everything you need to build production web applications in Rust</p>
            </div>
            <div class="grid grid-3">
                <div class="card">
                    <h3>📁 File-Based Routing</h3>
                    <p>Drop a <code>.rs</code> file in <code>pages/</code> and it becomes a route. Dynamic params with <code>[slug].rs</code>, catch-all with <code>[...path].rs</code>.</p>
                </div>
                <div class="card">
                    <h3>⚡ Multiple Render Modes</h3>
                    <p>Choose SSR, SSG, CSR, or ISR per page. Declare with <code>#[page(mode = "ssr")]</code>.</p>
                </div>
                <div class="card">
                    <h3>🔌 API Routes</h3>
                    <p>Co-located API endpoints in <code>api/</code> with typed JSON, form parsing, and method macros.</p>
                </div>
                <div class="card">
                    <h3>🔒 Auth & Sessions</h3>
                    <p>Built-in session management, role-based guards, and middleware for page and API protection.</p>
                </div>
                <div class="card">
                    <h3>🧩 Layouts & Components</h3>
                    <p>Nested layouts with <code>_layout.rs</code>, shared components, and section modules.</p>
                </div>
                <div class="card">
                    <h3>🛠 CLI Tooling</h3>
                    <p><code>vega new</code>, <code>vega dev</code>, <code>vega build</code>, <code>vega routes</code> — zero-config development.</p>
                </div>
            </div>
        </section>

        <section class="section">
            <div class="section-title">
                <h2>How It Works</h2>
                <p>Build-time scanning → compile-time codegen → runtime routing</p>
            </div>
            <div class="grid grid-2">
                <div class="card">
                    <h3>1. Scan</h3>
                    <p>At <code>cargo build</code>, Vega scans <code>pages/</code> and <code>api/</code> directories, parsing filenames into route segments.</p>
                </div>
                <div class="card">
                    <h3>2. Generate</h3>
                    <p>The router emits Rust code: module declarations, route tables, and runtime registration functions.</p>
                </div>
                <div class="card">
                    <h3>3. Compile</h3>
                    <p>Your handlers, middleware, and the generated glue code compile together into a single binary.</p>
                </div>
                <div class="card">
                    <h3>4. Serve</h3>
                    <p>At runtime, Axum serves requests using the compiled route table. No filesystem lookups.</p>
                </div>
            </div>
        </section>

        <section class="section">
            <div class="section-title">
                <h2>Built With</h2>
            </div>
            <div class="grid grid-3">
                <div class="card stat">
                    <div class="stat-value">🦀</div>
                    <div class="stat-label">Rust</div>
                </div>
                <div class="card stat">
                    <div class="stat-value">⚙️</div>
                    <div class="stat-label">Axum + Tower</div>
                </div>
                <div class="card stat">
                    <div class="stat-value">🔄</div>
                    <div class="stat-label">Tokio Runtime</div>
                </div>
            </div>
        </section>

        <section class="section" style="text-align:center">
            <h2>Try the Demo</h2>
            <p>Explore the blog, test auth flows, and inspect the API.</p>
            <div style="display:flex;gap:0.75rem;justify-content:center;margin-top:1rem">
                <a href="/blog" class="btn btn-primary">Browse Blog</a>
                <a href="/api/_vega/routes" class="btn btn-outline">View API Routes</a>
                <a href="/register" class="btn btn-outline">Create Account</a>
            </div>
        </section>"#,
        hero = crate::sections::hero::render(),
    )
}

pub async fn handler(State(state): State<AppState>, req: Request) -> Html<String> {
    let user = current_user_from_headers(req.headers(), &state).await;
    let body = render();
    Html(crate::pages::_layout::render_layout(
        "Home",
        &body,
        user.as_ref(),
        None,
    ))
}
