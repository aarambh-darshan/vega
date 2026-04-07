pub fn post_card(slug: &str, title: &str, excerpt: &str, tags: &[String]) -> String {
    format!(
        "<article class=\"card\">\
            <h3><a href=\"/blog/{}\">{}</a></h3>\
            <p>{}</p>\
            <p><strong>Tags:</strong> {}</p>\
         </article>",
        crate::app::esc(slug),
        crate::app::esc(title),
        crate::app::esc(excerpt),
        crate::app::esc(&tags.join(", "))
    )
}

pub fn list_row(id: u64, title: &str, slug: &str, tags: &[String]) -> String {
    format!(
        "<li>#{id} <strong>{}</strong> <code>{}</code> <small>[{}]</small></li>",
        crate::app::esc(title),
        crate::app::esc(slug),
        crate::app::esc(&tags.join(", "))
    )
}
