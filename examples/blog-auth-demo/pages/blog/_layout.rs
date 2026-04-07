#[vega::layout]
pub fn Layout(children: fn() -> &'static str) -> &'static str {
    let _ = children;
    "Blog Layout"
}

pub fn render_blog_layout(content: &str) -> String {
    content.to_string()
}
