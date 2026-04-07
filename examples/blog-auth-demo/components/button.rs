/// Render a styled link button.
pub fn link(text: &str, href: &str) -> String {
    format!(
        r#"<a href="{href}" class="btn btn-outline btn-sm">{text}</a>"#,
        href = vega::core::html_escape(href),
        text = vega::core::html_escape(text),
    )
}

/// Render a primary link button.
pub fn link_primary(text: &str, href: &str) -> String {
    format!(
        r#"<a href="{href}" class="btn btn-primary btn-sm">{text}</a>"#,
        href = vega::core::html_escape(href),
        text = vega::core::html_escape(text),
    )
}

/// Render a submit button for forms.
pub fn submit(text: &str) -> String {
    format!(
        r#"<button type="submit" class="btn btn-primary">{text}</button>"#,
        text = vega::core::html_escape(text),
    )
}
