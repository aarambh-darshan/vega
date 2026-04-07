pub fn render() -> &'static str {
    r#"<div class="hero">
        <h1>Build <span>fast</span>, ship <span>faster</span></h1>
        <p>Vega is a Next.js-inspired web framework for Rust — file-based routing, SSR, typed APIs, and zero-config tooling.</p>
        <div style="display:flex;gap:0.75rem;justify-content:center">
            <a href="/blog" class="btn btn-primary">Explore Blog Demo</a>
            <a href="/docs" class="btn btn-outline">Read Docs</a>
        </div>
    </div>"#
}
