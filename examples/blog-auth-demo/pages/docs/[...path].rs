use crate::app::{current_user_from_headers, AppState};
use vega::server::web::{Html, Path, Request, State};

#[vega::page(mode = "ssg")]
pub fn DocsPage() -> &'static str {
    "Docs catch-all"
}

pub fn render(path: &str) -> String {
    format!(
        r#"<div style="margin-bottom:0.75rem">
            <a href="/docs" style="font-size:0.875rem;color:var(--ink-muted)">← Back to Docs</a>
        </div>
        <h1>Docs — Catch-All Route</h1>
        <div class="card">
            <p>This page demonstrates the <code>[...path]</code> catch-all route segment.</p>
            <p>URL path captured: <code>{path}</code></p>
            <p>The filename is <code>pages/docs/[...path].rs</code>, which matches any nested path under <code>/docs/</code>.</p>
        </div>
        <div class="card">
            <h3>Try more paths</h3>
            <ul style="padding-left:1.25rem;color:var(--ink-muted)">
                <li><a href="/docs/getting-started">/docs/getting-started</a></li>
                <li><a href="/docs/routing/dynamic">/docs/routing/dynamic</a></li>
                <li><a href="/docs/api/auth/session">/docs/api/auth/session</a></li>
            </ul>
        </div>"#,
        path = vega::core::html_escape(path)
    )
}

pub async fn handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    req: Request,
) -> Html<String> {
    let user = current_user_from_headers(req.headers(), &state).await;
    let body = render(&path);
    Html(crate::pages::_layout::render_layout(
        &format!("Docs — {}", path),
        &body,
        user.as_ref(),
        None,
    ))
}
