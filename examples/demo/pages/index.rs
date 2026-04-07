use vega::server::web::{Html, Request};

#[vega::page(mode = "ssr")]
pub fn IndexPage() -> &'static str {
    "Hello from Vega"
}

pub async fn handler(_req: Request) -> Html<String> {
    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Welcome — Vega</title>
</head>
<body>
    <h1>Welcome to Vega 🚀</h1>
</body>
</html>"#
    ))
}
