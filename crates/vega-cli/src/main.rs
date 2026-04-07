use anyhow::{bail, Context};
use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Parser)]
#[command(name = "vega", version, about = "Vega CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    New {
        name: String,
        #[arg(long)]
        tailwind: bool,
        #[arg(long)]
        auth: bool,
        #[arg(long)]
        db: Option<String>,
    },
    Dev {
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    Build {
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    Generate {
        kind: GenerateKind,
        name: String,
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    Routes {
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum GenerateKind {
    Page,
    Api,
    Component,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            tailwind,
            auth,
            db,
        } => scaffold_new(&name, tailwind, auth, db.as_deref())?,
        Commands::Dev { path } => run_dev(path).await?,
        Commands::Build { path } => run_build(path).await?,
        Commands::Generate { kind, name, path } => run_generate(kind, &name, &path)?,
        Commands::Routes { path } => run_routes(path)?,
    }

    Ok(())
}

fn scaffold_new(name: &str, tailwind: bool, auth: bool, db: Option<&str>) -> anyhow::Result<()> {
    let root = PathBuf::from(name);
    if root.exists() {
        bail!("target directory already exists: {}", root.display());
    }

    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("pages"))?;
    fs::create_dir_all(root.join("api"))?;
    fs::create_dir_all(root.join("components"))?;
    fs::create_dir_all(root.join("sections"))?;
    fs::create_dir_all(root.join("styles"))?;
    fs::create_dir_all(root.join("public"))?;

    fs::write(
        root.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
build = "build.rs"

[features]
default = ["ssr"]
ssr = ["vega/ssr"]

[dependencies]
vega = {{ version = "0.9", features = ["ssr"] }}
anyhow = "1"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"

[build-dependencies]
anyhow = "1"
vega-router = "0.9"
"#
        ),
    )?;

    let mut extra = String::new();
    if let Some(db) = db {
        extra.push_str(&format!("\n[database]\nurl = \"{db}\"\n"));
    }
    if auth {
        extra.push_str("\n[auth]\nprovider = \"session\"\n");
    }

    fs::write(
        root.join("Vega.toml"),
        format!(
            r#"[app]
name = "{name}"
base_url = "http://localhost:3000"

[server]
host = "0.0.0.0"
port = 3000

[build]
out_dir = "dist"
public_dir = "public"

[features]
tailwind = {tailwind}
compress = false
source_maps = true

[ssr]
streaming = true

[ssg]
concurrent = 4
{extra}"#
        ),
    )?;

    fs::write(
        root.join("build.rs"),
        r#"fn main() {
    println!("cargo:rerun-if-changed=pages/");
    println!("cargo:rerun-if-changed=api/");
    println!("cargo:rerun-if-changed=components/");
    println!("cargo:rerun-if-changed=sections/");
    println!("cargo:rerun-if-changed=app.rs");
    println!("cargo:rerun-if-changed=middleware.rs");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");
    let out = std::path::Path::new(&out_dir);
    vega_router::generate_all(".", out).expect("vega codegen");
}
"#,
    )?;

    // One-liner main.rs — everything is auto-generated
    fs::write(
        root.join("src/main.rs"),
        "#![allow(dead_code, non_snake_case)]\ninclude!(concat!(env!(\"OUT_DIR\"), \"/main_gen.rs\"));\n",
    )?;

    fs::write(
        root.join("pages/index.rs"),
        r##"use vega::server::web::{Html, Request};

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
    <style>
        body {{ font-family: system-ui, sans-serif; margin: 0; padding: 4rem 2rem; text-align: center; background: #fafaf9; color: #1c1917; }}
        h1 {{ font-size: 2.5rem; margin-bottom: 0.5rem; }}
        h1 span {{ color: #0d9488; }}
        p {{ color: #78716c; font-size: 1.125rem; }}
        code {{ background: #f5f5f4; padding: 0.25rem 0.5rem; border-radius: 6px; font-size: 0.875rem; }}
    </style>
</head>
<body>
    <h1>Welcome to <span>Vega</span> 🚀</h1>
    <p>Edit <code>pages/index.rs</code> to get started.</p>
</body>
</html>"#
    ))
}
"##,
    )?;

    fs::write(
        root.join("styles/global.css"),
        "body { font-family: sans-serif; margin: 0; padding: 2rem; }\n",
    )?;

    println!("\u{2728} Created project: {}", root.display());
    println!();
    println!("  cd {name}");
    println!("  cargo run        # or: vega dev");
    println!();
    println!("  Visit http://localhost:3000");
    Ok(())
}

async fn run_dev(path: PathBuf) -> anyhow::Result<()> {
    let status = Command::new("cargo")
        .arg("run")
        .current_dir(path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await
        .context("failed to run cargo run")?;

    if !status.success() {
        bail!("dev command failed");
    }

    Ok(())
}

async fn run_build(path: PathBuf) -> anyhow::Result<()> {
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await
        .context("failed to run cargo build --release")?;

    if !status.success() {
        bail!("build command failed");
    }

    // Basic SSG output for static routes.
    let manifest = vega_router::scan_project(&path)?;
    let dist = path.join("dist");
    fs::create_dir_all(&dist)?;

    for route in manifest
        .pages
        .into_iter()
        .filter(|r| !r.route_path.contains(':') && !r.route_path.contains('*'))
    {
        let relative = if route.route_path == "/" {
            PathBuf::from("index.html")
        } else {
            PathBuf::from(route.route_path.trim_start_matches('/')).join("index.html")
        };
        let html_path = dist.join(relative);
        if let Some(parent) = html_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(
            html_path,
            format!(
                "<!doctype html><html><body><h1>SSG {}</h1><p>{}</p></body></html>",
                route.route_path, route.module_path
            ),
        )?;
    }

    println!("build completed: {}", dist.display());
    Ok(())
}

fn run_generate(kind: GenerateKind, name: &str, root: &Path) -> anyhow::Result<()> {
    if name.trim().is_empty() {
        bail!("name cannot be empty");
    }

    let file_name = if name.ends_with(".rs") {
        name.to_string()
    } else {
        format!("{name}.rs")
    };

    let (dir, content) = match kind {
        GenerateKind::Page => (
            root.join("pages"),
            format!(
                "#[vega::page(mode = \"ssr\")]\npub fn {}() -> &'static str {{\n    \"{} page\"\n}}\n",
                to_component_name(name),
                name
            ),
        ),
        GenerateKind::Api => (
            root.join("api"),
            "#[vega::get]\npub async fn handler() -> vega::server::ApiResponse {\n    vega::server::ApiResponse::json(serde_json::json!({\"ok\": true}))\n}\n".to_string(),
        ),
        GenerateKind::Component => (
            root.join("components"),
            format!("pub fn {}() -> &'static str {{\n    \"component\"\n}}\n", to_component_name(name)),
        ),
    };

    fs::create_dir_all(&dir)?;
    let target = dir.join(file_name);
    if target.exists() {
        bail!("file already exists: {}", target.display());
    }

    fs::write(&target, content)?;
    println!("generated {}", target.display());
    Ok(())
}

fn run_routes(root: PathBuf) -> anyhow::Result<()> {
    let manifest = vega_router::scan_project(&root)?;

    println!("Pages:");
    for page in manifest.pages.iter().filter(|page| !page.is_special) {
        println!("  {} -> {}", page.route_path, page.module_path);
    }

    println!("API:");
    for api in &manifest.api {
        let methods = api
            .methods
            .iter()
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        println!("  [{}] {} -> {}", methods, api.route_path, api.module_path);
    }

    Ok(())
}

fn to_component_name(input: &str) -> String {
    input
        .split(['-', '_', '/'])
        .filter(|s| !s.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => {
                    format!(
                        "{}{}",
                        first.to_ascii_uppercase(),
                        chars.as_str().to_ascii_lowercase()
                    )
                }
                None => String::new(),
            }
        })
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_name_conversion() {
        assert_eq!(to_component_name("about"), "About");
        assert_eq!(to_component_name("blog_post"), "BlogPost");
        assert_eq!(to_component_name("admin/users"), "AdminUsers");
    }
}
