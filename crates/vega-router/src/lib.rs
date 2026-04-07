use anyhow::Context;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;
use vega_core::{ApiRouteEntry, HttpMethod, RouteEntry, RouteManifest, SegmentKind};
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum RouterError {
    #[error("{0}")]
    Generic(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy)]
enum ScanRoot {
    Pages,
    Api,
    Components,
    Sections,
}

impl ScanRoot {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pages => "pages",
            Self::Api => "api",
            Self::Components => "components",
            Self::Sections => "sections",
        }
    }
}

/// Detected convention files in the project root.
#[derive(Debug, Clone)]
pub struct ProjectConventions {
    /// `true` if `app.rs` exists at the project root.
    pub has_app: bool,
    /// `true` if `middleware.rs` exists at the project root.
    pub has_middleware: bool,
    /// Absolute path to `app.rs` (if it exists).
    pub app_path: Option<PathBuf>,
    /// Absolute path to `middleware.rs` (if it exists).
    pub middleware_path: Option<PathBuf>,
    /// Middleware functions detected in middleware.rs.
    pub middleware_fns: Vec<String>,
    /// Pages that have a `fn post_handler(` — (route_path, module_path).
    pub page_post_handlers: Vec<(String, String)>,
}

impl ProjectConventions {
    /// The Rust type path for the application state.
    pub fn state_type(&self) -> &str {
        if self.has_app {
            "crate::app::AppState"
        } else {
            "()"
        }
    }
}

/// Detect convention files at the project root.
pub fn detect_conventions(root: &Path, pages: &[RouteEntry]) -> anyhow::Result<ProjectConventions> {
    let app_file = root.join("app.rs");
    let mw_file = root.join("middleware.rs");
    let has_app = app_file.exists();
    let has_middleware = mw_file.exists();

    let app_path = if has_app {
        Some(fs::canonicalize(&app_file)?)
    } else {
        None
    };
    let middleware_path = if has_middleware {
        Some(fs::canonicalize(&mw_file)?)
    } else {
        None
    };

    let middleware_fns = if has_middleware {
        let content = fs::read_to_string(&mw_file)?;
        let mut fns = Vec::new();
        for name in &[
            "require_auth_page",
            "require_admin_page",
            "require_auth_api",
            "require_admin_api",
        ] {
            if source_has_named_fn_in_text(&content, name) {
                fns.push(name.to_string());
            }
        }
        fns
    } else {
        Vec::new()
    };

    // Scan pages for post_handler functions
    let pages_root = root.join("pages");
    let mut page_post_handlers = Vec::new();
    for entry in pages.iter().filter(|e| !e.is_special) {
        let file = pages_root.join(&entry.file_path);
        if source_has_named_fn(&file, "post_handler").unwrap_or(false) {
            page_post_handlers.push((entry.route_path.clone(), entry.module_path.clone()));
        }
    }

    Ok(ProjectConventions {
        has_app,
        has_middleware,
        app_path,
        middleware_path,
        middleware_fns,
        page_post_handlers,
    })
}

#[derive(Debug, Default, Clone)]
struct ModuleNode {
    children: BTreeMap<String, ModuleNode>,
    files: Vec<(String, PathBuf)>,
}

pub fn parse_segment(input: &str) -> SegmentKind {
    match input {
        "index" => SegmentKind::Index,
        "_layout" => SegmentKind::Layout,
        "_error" => SegmentKind::Error,
        "_loading" => SegmentKind::Loading,
        "_not_found" => SegmentKind::NotFound,
        _ => {
            if let Some(name) = input.strip_prefix("[...").and_then(|v| v.strip_suffix(']')) {
                SegmentKind::CatchAll(name.to_string())
            } else if let Some(name) = input.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
                SegmentKind::Dynamic(name.to_string())
            } else if let Some(name) = input.strip_prefix('(').and_then(|v| v.strip_suffix(')')) {
                SegmentKind::Group(name.to_string())
            } else {
                SegmentKind::Static(input.to_string())
            }
        }
    }
}

pub fn sanitize_module_name(input: &str) -> String {
    match parse_segment(input) {
        SegmentKind::Dynamic(name) => format!("{}_dynamic", sanitize_ident(&name)),
        SegmentKind::CatchAll(name) => format!("{}_catchall", sanitize_ident(&name)),
        SegmentKind::Group(name) => format!("{}_group", sanitize_ident(&name)),
        SegmentKind::Static(name) => sanitize_ident(&name),
        SegmentKind::Index => "index".to_string(),
        SegmentKind::Layout => "_layout".to_string(),
        SegmentKind::Error => "_error".to_string(),
        SegmentKind::Loading => "_loading".to_string(),
        SegmentKind::NotFound => "_not_found".to_string(),
    }
}

fn sanitize_ident(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .to_ascii_lowercase()
}

pub fn scan_project(root: impl AsRef<Path>) -> anyhow::Result<RouteManifest> {
    let root = root.as_ref();
    let pages = scan_pages(root.join("pages"))?;
    let api = scan_api(root.join("api"))?;
    Ok(RouteManifest { pages, api })
}

pub fn scan_pages(root: impl AsRef<Path>) -> anyhow::Result<Vec<RouteEntry>> {
    let root = root.as_ref();
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for file in rust_files(root)? {
        let rel = file.strip_prefix(root).context("invalid page path")?;
        let entry = route_entry_from_rel(rel, ScanRoot::Pages)?;
        entries.push(entry);
    }

    let layout_map = collect_layouts(&entries);
    for entry in entries.iter_mut().filter(|e| !e.is_special) {
        entry.layouts = layout_chain_for(entry, &layout_map);
    }

    entries.sort_by(|a, b| {
        a.route_path
            .cmp(&b.route_path)
            .then(a.module_path.cmp(&b.module_path))
    });
    Ok(entries)
}

pub fn scan_api(root: impl AsRef<Path>) -> anyhow::Result<Vec<ApiRouteEntry>> {
    let root = root.as_ref();
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for file in rust_files(root)? {
        let rel = file.strip_prefix(root).context("invalid api path")?;
        let page_like = route_entry_from_rel(rel, ScanRoot::Api)?;
        if page_like.is_special {
            continue;
        }

        let methods = infer_api_methods(&file)?;
        entries.push(ApiRouteEntry {
            file_path: page_like.file_path,
            route_path: format!("/api{}", page_like.route_path),
            module_path: page_like.module_path,
            methods,
        });
    }

    entries.sort_by(|a, b| {
        a.route_path
            .cmp(&b.route_path)
            .then(a.module_path.cmp(&b.module_path))
    });
    Ok(entries)
}

fn rust_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }

        files.push(path.to_path_buf());
    }
    Ok(files)
}

fn infer_api_methods(file_path: &Path) -> anyhow::Result<Vec<HttpMethod>> {
    let content = fs::read_to_string(file_path)?;
    let mut methods = Vec::new();
    let checks = [
        ("get", HttpMethod::Get),
        ("post", HttpMethod::Post),
        ("put", HttpMethod::Put),
        ("patch", HttpMethod::Patch),
        ("delete", HttpMethod::Delete),
    ];

    for (method_name, method) in checks {
        let plain = format!("#[{method_name}");
        let namespaced = format!("#[vega::{method_name}");
        if content.contains(&plain) || content.contains(&namespaced) {
            methods.push(method);
        }
    }

    if methods.is_empty() {
        methods.push(HttpMethod::Get);
    }

    Ok(methods)
}

fn route_entry_from_rel(rel_path: &Path, root: ScanRoot) -> anyhow::Result<RouteEntry> {
    let mut dir_segments = Vec::new();
    let mut module_segments = Vec::new();

    let parent = rel_path.parent().unwrap_or_else(|| Path::new(""));
    for segment in parent.components() {
        if let Component::Normal(v) = segment {
            let seg = v
                .to_str()
                .ok_or_else(|| RouterError::Generic("non utf-8 path segment".to_string()))?;
            match parse_segment(seg) {
                SegmentKind::Group(_) => module_segments.push(sanitize_module_name(seg)),
                SegmentKind::Dynamic(name) => {
                    dir_segments.push(format!(":{name}"));
                    module_segments.push(sanitize_module_name(seg));
                }
                SegmentKind::CatchAll(name) => {
                    dir_segments.push(format!("*{name}"));
                    module_segments.push(sanitize_module_name(seg));
                }
                _ => {
                    dir_segments.push(seg.to_string());
                    module_segments.push(sanitize_module_name(seg));
                }
            }
        }
    }

    let file_stem = rel_path
        .file_stem()
        .and_then(|v| v.to_str())
        .ok_or_else(|| RouterError::Generic("invalid file stem".to_string()))?;

    let file_segment = parse_segment(file_stem);
    let is_special = file_segment.is_special_file();

    let route_tail = match file_segment {
        SegmentKind::Index => None,
        SegmentKind::Dynamic(ref value) => Some(format!(":{value}")),
        SegmentKind::CatchAll(ref value) => Some(format!("*{value}")),
        SegmentKind::Group(_) => None,
        SegmentKind::Static(ref value) => Some(value.clone()),
        SegmentKind::Layout | SegmentKind::Error | SegmentKind::Loading | SegmentKind::NotFound => {
            None
        }
    };

    let mut route_parts = dir_segments.clone();
    if let Some(route_tail) = route_tail {
        route_parts.push(route_tail);
    }

    let mut module_parts = vec![root.as_str().to_string()];
    module_parts.extend(module_segments);
    module_parts.push(sanitize_module_name(file_stem));

    let route_path = if route_parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", route_parts.join("/"))
    };

    let directory_for_layout = if parent.components().count() == 0 {
        "".to_string()
    } else {
        parent.to_string_lossy().replace('\\', "/")
    };

    Ok(RouteEntry {
        file_path: rel_path.to_string_lossy().replace('\\', "/"),
        route_path,
        module_path: module_parts.join("::"),
        is_special,
        layouts: vec![directory_for_layout],
    })
}

fn collect_layouts(entries: &[RouteEntry]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for entry in entries {
        if !entry.module_path.ends_with("::_layout") {
            continue;
        }

        let dir = entry
            .file_path
            .rsplit_once('/')
            .map(|(a, _)| a)
            .unwrap_or("");
        map.insert(dir.to_string(), entry.module_path.clone());
    }
    map
}

fn layout_chain_for(entry: &RouteEntry, layout_map: &BTreeMap<String, String>) -> Vec<String> {
    let dir = entry
        .file_path
        .rsplit_once('/')
        .map(|(a, _)| a)
        .unwrap_or("");
    let mut chain = Vec::new();

    if let Some(layout) = layout_map.get("") {
        chain.push(layout.clone());
    }

    let mut current = String::new();
    for segment in dir.split('/') {
        if segment.is_empty() {
            continue;
        }
        if !current.is_empty() {
            current.push('/');
        }
        current.push_str(segment);
        if let Some(layout) = layout_map.get(&current) {
            chain.push(layout.clone());
        }
    }

    chain
}

pub fn generate_all(
    project_root: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let root = project_root.as_ref();
    let out = out_dir.as_ref();
    fs::create_dir_all(out)?;

    generate_pages_mod(root.join("pages"), out.join("pages_mod.rs"))?;
    generate_api_mod(root.join("api"), out.join("api_mod.rs"))?;
    generate_components_mod(root.join("components"), out.join("components_mod.rs"))?;
    generate_sections_mod(root.join("sections"), out.join("sections_mod.rs"))?;

    let pages = scan_pages(root.join("pages"))?;
    let api = scan_api(root.join("api"))?;

    // Detect convention files
    let conventions = detect_conventions(root, &pages)?;

    generate_router_file(&pages, out.join("router.rs"))?;
    generate_api_router_file(&api, out.join("api_router.rs"))?;
    generate_page_runtime_routes_file(
        root.join("pages"),
        &pages,
        out.join("runtime_pages.rs"),
        &conventions,
    )?;
    generate_api_runtime_routes_file(
        root.join("api"),
        &api,
        out.join("runtime_api.rs"),
        &conventions,
    )?;
    generate_main_file(root, out.join("main_gen.rs"), &conventions)?;

    Ok(())
}

pub fn generate_pages_mod(
    root_dir: impl AsRef<Path>,
    out_file: impl AsRef<Path>,
) -> anyhow::Result<()> {
    generate_mod_tree(root_dir, out_file, ScanRoot::Pages)
}

pub fn generate_api_mod(
    root_dir: impl AsRef<Path>,
    out_file: impl AsRef<Path>,
) -> anyhow::Result<()> {
    generate_mod_tree(root_dir, out_file, ScanRoot::Api)
}

pub fn generate_components_mod(
    root_dir: impl AsRef<Path>,
    out_file: impl AsRef<Path>,
) -> anyhow::Result<()> {
    generate_mod_tree(root_dir, out_file, ScanRoot::Components)
}

pub fn generate_sections_mod(
    root_dir: impl AsRef<Path>,
    out_file: impl AsRef<Path>,
) -> anyhow::Result<()> {
    generate_mod_tree(root_dir, out_file, ScanRoot::Sections)
}

fn generate_mod_tree(
    root_dir: impl AsRef<Path>,
    out_file: impl AsRef<Path>,
    root: ScanRoot,
) -> anyhow::Result<()> {
    let root_dir = root_dir.as_ref();
    if !root_dir.exists() {
        fs::write(
            out_file,
            format!("// AUTO-GENERATED\npub mod {} {{}}\n", root.as_str()),
        )?;
        return Ok(());
    }

    let mut tree = ModuleNode::default();
    let mut duplicates = BTreeSet::new();

    for file in rust_files(root_dir)? {
        let rel = file.strip_prefix(root_dir)?;
        let rel_parent = rel.parent().unwrap_or_else(|| Path::new(""));

        let mut cursor = &mut tree;
        for segment in rel_parent.components() {
            if let Component::Normal(name) = segment {
                let name = name.to_string_lossy();
                let sanitized = sanitize_module_name(&name);
                cursor = cursor.children.entry(sanitized).or_default();
            }
        }

        let stem = rel
            .file_stem()
            .and_then(|v| v.to_str())
            .ok_or_else(|| RouterError::Generic("invalid file stem".to_string()))?;
        let file_mod = sanitize_module_name(stem);

        let key = format!("{}:{}", rel_parent.display(), file_mod);
        if !duplicates.insert(key) {
            return Err(anyhow::anyhow!(
                "duplicate sanitized module name detected in {} for {}",
                rel_parent.display(),
                file_mod
            ));
        }

        let abs_file = fs::canonicalize(&file)?;
        cursor.files.push((file_mod, abs_file));
    }

    let mut output = String::new();
    output.push_str("// AUTO-GENERATED BY vega-router. DO NOT EDIT.\n");
    output.push_str(&format!("pub mod {} {{\n", root.as_str()));
    write_module_node(&tree, 1, &mut output);
    output.push_str("}\n");
    fs::write(out_file, output)?;
    Ok(())
}

fn write_module_node(node: &ModuleNode, depth: usize, out: &mut String) {
    let indent = "    ".repeat(depth);

    for (file_mod, abs_path) in &node.files {
        out.push_str(&format!(
            "{indent}#[path = \"{}\"]\n{indent}pub mod {file_mod};\n",
            abs_path.display()
        ));
    }

    for (name, child) in &node.children {
        out.push_str(&format!("{indent}pub mod {name} {{\n"));
        write_module_node(child, depth + 1, out);
        out.push_str(&format!("{indent}}}\n"));
    }
}

pub fn generate_router_file(
    entries: &[RouteEntry],
    out_file: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let mut output = String::new();
    output.push_str("// AUTO-GENERATED BY vega-router. DO NOT EDIT.\n");
    output.push_str("pub fn vega_generated_page_routes() -> Vec<vega::core::RouteEntry> {\n");
    output.push_str("    vec![\n");

    for entry in entries.iter().filter(|e| !e.is_special) {
        let layouts_literal = entry
            .layouts
            .iter()
            .map(|layout| format!("\"{}\".to_string()", layout))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "        vega::core::RouteEntry {{ file_path: \"{}\".to_string(), route_path: \"{}\".to_string(), module_path: \"{}\".to_string(), is_special: false, layouts: vec![{}] }},\n",
            entry.file_path, entry.route_path, entry.module_path, layouts_literal
        ));
    }

    output.push_str("    ]\n");
    output.push_str("}\n");
    fs::write(out_file, output)?;
    Ok(())
}

pub fn generate_api_router_file(
    entries: &[ApiRouteEntry],
    out_file: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let mut output = String::new();
    output.push_str("// AUTO-GENERATED BY vega-router. DO NOT EDIT.\n");
    output.push_str("pub fn vega_generated_api_routes() -> Vec<vega::core::ApiRouteEntry> {\n");
    output.push_str("    vec![\n");

    for entry in entries {
        let methods = entry
            .methods
            .iter()
            .map(|method| format!("vega::core::HttpMethod::{}", method_variant_name(*method)))
            .collect::<Vec<_>>()
            .join(", ");

        output.push_str(&format!(
            "        vega::core::ApiRouteEntry {{ file_path: \"{}\".to_string(), route_path: \"{}\".to_string(), module_path: \"{}\".to_string(), methods: vec![{}] }},\n",
            entry.file_path, entry.route_path, entry.module_path, methods
        ));
    }

    output.push_str("    ]\n");
    output.push_str("}\n");
    fs::write(out_file, output)?;
    Ok(())
}

pub fn generate_page_runtime_routes_file(
    pages_root: impl AsRef<Path>,
    entries: &[RouteEntry],
    out_file: impl AsRef<Path>,
    conventions: &ProjectConventions,
) -> anyhow::Result<()> {
    let pages_root = pages_root.as_ref();
    let state_type = conventions.state_type();
    let has_auth = conventions
        .middleware_fns
        .contains(&"require_auth_page".to_string());
    let has_admin = conventions
        .middleware_fns
        .contains(&"require_admin_page".to_string());

    let mut output = String::new();
    output.push_str("// AUTO-GENERATED BY vega-router. DO NOT EDIT.\n");
    output.push_str(
        "#[allow(unused_mut, unused_variables, clippy::unit_arg, clippy::clone_on_copy)]\n",
    );
    output.push_str("pub fn vega_register_page_runtime_routes(\n");
    output.push_str(&format!(
        "    mut app: vega::server::web::Router<{}>,\n",
        state_type
    ));
    output.push_str(&format!("    state: {},\n", state_type));
    output.push_str(&format!(
        ") -> vega::server::web::Router<{}> {{\n",
        state_type
    ));

    if has_auth {
        output.push_str("    let auth_page = vega::server::web::from_fn_with_state(state.clone(), crate::middleware::require_auth_page);\n");
    }
    if has_admin {
        output.push_str("    let admin_page = vega::server::web::from_fn_with_state(state.clone(), crate::middleware::require_admin_page);\n");
    }

    for entry in entries.iter().filter(|entry| !entry.is_special) {
        let source_file = pages_root.join(&entry.file_path);
        let has_submit = source_has_named_fn(&source_file, "submit")?;
        let route_path = to_axum_path(&entry.route_path);

        let mut method_router = format!(
            "vega::server::web::get(crate::{}::handler)",
            entry.module_path
        );
        if has_submit {
            method_router.push_str(&format!(".post(crate::{}::submit)", entry.module_path));
        }

        if has_admin && entry.route_path.starts_with("/admin") {
            method_router.push_str(".layer(admin_page.clone())");
        } else if has_auth && (entry.route_path == "/dashboard" || entry.route_path == "/logout") {
            method_router.push_str(".layer(auth_page.clone())");
        }

        output.push_str(&format!(
            "    app = app.route(\"{}\", {});\n",
            route_path, method_router
        ));
    }

    output.push_str("    app\n");
    output.push_str("}\n");
    fs::write(out_file, output)?;
    Ok(())
}

pub fn generate_api_runtime_routes_file(
    api_root: impl AsRef<Path>,
    entries: &[ApiRouteEntry],
    out_file: impl AsRef<Path>,
    conventions: &ProjectConventions,
) -> anyhow::Result<()> {
    let api_root = api_root.as_ref();
    let state_type = conventions.state_type();
    let has_auth_api = conventions
        .middleware_fns
        .contains(&"require_auth_api".to_string());
    let has_admin_api = conventions
        .middleware_fns
        .contains(&"require_admin_api".to_string());

    let mut output = String::new();
    output.push_str("// AUTO-GENERATED BY vega-router. DO NOT EDIT.\n");
    output.push_str(
        "#[allow(unused_mut, unused_variables, clippy::unit_arg, clippy::clone_on_copy)]\n",
    );
    output.push_str("pub fn vega_register_api_runtime_routes(\n");
    output.push_str(&format!(
        "    mut app: vega::server::web::Router<{}>,\n",
        state_type
    ));
    output.push_str(&format!("    state: {},\n", state_type));
    output.push_str(&format!(
        ") -> vega::server::web::Router<{}> {{\n",
        state_type
    ));

    if has_auth_api {
        output.push_str("    let auth_api = vega::server::web::from_fn_with_state(state.clone(), crate::middleware::require_auth_api);\n");
    }
    if has_admin_api {
        output.push_str("    let admin_api = vega::server::web::from_fn_with_state(state.clone(), crate::middleware::require_admin_api);\n");
    }

    for entry in entries {
        if entry.methods.is_empty() {
            continue;
        }

        let source_file = api_root.join(&entry.file_path);
        let ordered_methods = ordered_methods(&entry.methods);
        if ordered_methods.is_empty() {
            continue;
        }

        let mut method_iter = ordered_methods.into_iter();
        let first = method_iter
            .next()
            .ok_or_else(|| anyhow::anyhow!("expected at least one HTTP method"))?;

        let first_handler = resolve_api_handler_name(&source_file, first)?;
        let mut method_router = format!(
            "vega::server::web::{}(crate::{}::{})",
            method_builder_name(first),
            entry.module_path,
            first_handler
        );

        for method in method_iter {
            let handler_name = resolve_api_handler_name(&source_file, method)?;
            method_router.push_str(&format!(
                ".{}(crate::{}::{})",
                method_builder_name(method),
                entry.module_path,
                handler_name
            ));
        }

        if has_admin_api && entry.route_path.starts_with("/api/admin") {
            method_router.push_str(".layer(admin_api.clone())");
        } else if has_auth_api
            && (entry.route_path == "/api/auth/me" || entry.route_path == "/api/auth/logout")
        {
            method_router.push_str(".layer(auth_api.clone())");
        }

        output.push_str(&format!(
            "    app = app.route(\"{}\", {});\n",
            to_axum_path(&entry.route_path),
            method_router
        ));
    }

    output.push_str("    app\n");
    output.push_str("}\n");
    fs::write(out_file, output)?;
    Ok(())
}

fn ordered_methods(methods: &[HttpMethod]) -> Vec<HttpMethod> {
    let order = [
        HttpMethod::Get,
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Patch,
        HttpMethod::Delete,
    ];

    let mut ordered = Vec::new();
    for candidate in order {
        if methods.contains(&candidate) {
            ordered.push(candidate);
        }
    }
    ordered
}

fn resolve_api_handler_name(file: &Path, method: HttpMethod) -> anyhow::Result<String> {
    let content = fs::read_to_string(file).with_context(|| {
        format!(
            "failed to read API source file for handler resolution: {}",
            file.display()
        )
    })?;

    let candidates: &[&str] = match method {
        HttpMethod::Get => &["handler", "get", "list", "me"],
        HttpMethod::Post => &[
            "create", "post", "submit", "login", "logout", "register", "handler",
        ],
        HttpMethod::Put => &["replace", "update", "put"],
        HttpMethod::Patch => &["patch", "update"],
        HttpMethod::Delete => &["delete", "remove"],
    };

    for candidate in candidates {
        if source_has_named_fn_in_text(&content, candidate) {
            return Ok((*candidate).to_string());
        }
    }

    anyhow::bail!(
        "no suitable handler function found for {} in {}",
        method.as_str(),
        file.display()
    );
}

fn source_has_named_fn(file: &Path, fn_name: &str) -> anyhow::Result<bool> {
    let content = fs::read_to_string(file)
        .with_context(|| format!("failed to read source file: {}", file.display()))?;
    Ok(source_has_named_fn_in_text(&content, fn_name))
}

fn source_has_named_fn_in_text(content: &str, fn_name: &str) -> bool {
    let needle = format!("fn {fn_name}(");
    content.contains(&needle)
}

fn method_builder_name(method: HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "get",
        HttpMethod::Post => "post",
        HttpMethod::Put => "put",
        HttpMethod::Patch => "patch",
        HttpMethod::Delete => "delete",
    }
}

fn to_axum_path(path: &str) -> String {
    let converted = path
        .split('/')
        .map(|segment| {
            if let Some(name) = segment.strip_prefix(':') {
                format!("{{{name}}}")
            } else if let Some(name) = segment.strip_prefix('*') {
                format!("{{*{name}}}")
            } else {
                segment.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("/");

    if converted.is_empty() {
        "/".to_string()
    } else if converted.starts_with('/') {
        converted
    } else {
        format!("/{converted}")
    }
}

fn method_variant_name(method: HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "Get",
        HttpMethod::Post => "Post",
        HttpMethod::Put => "Put",
        HttpMethod::Patch => "Patch",
        HttpMethod::Delete => "Delete",
    }
}

/// Generate the `main_gen.rs` file that contains the full application entry point.
///
/// This file is `include!()`-d from `src/main.rs` so the user never has to
/// write boilerplate server setup code.
pub fn generate_main_file(
    _project_root: &Path,
    out_file: impl AsRef<Path>,
    conventions: &ProjectConventions,
) -> anyhow::Result<()> {
    let mut output = String::new();
    output.push_str("// AUTO-GENERATED BY vega-router. DO NOT EDIT.\n");
    output.push('\n');

    // Include generated modules
    output.push_str("include!(concat!(env!(\"OUT_DIR\"), \"/pages_mod.rs\"));\n");
    output.push_str("include!(concat!(env!(\"OUT_DIR\"), \"/api_mod.rs\"));\n");
    output.push_str("include!(concat!(env!(\"OUT_DIR\"), \"/components_mod.rs\"));\n");
    output.push_str("include!(concat!(env!(\"OUT_DIR\"), \"/sections_mod.rs\"));\n");
    output.push_str("include!(concat!(env!(\"OUT_DIR\"), \"/router.rs\"));\n");
    output.push_str("include!(concat!(env!(\"OUT_DIR\"), \"/api_router.rs\"));\n");
    output.push_str("include!(concat!(env!(\"OUT_DIR\"), \"/runtime_pages.rs\"));\n");
    output.push_str("include!(concat!(env!(\"OUT_DIR\"), \"/runtime_api.rs\"));\n\n");

    // Include convention files
    if let Some(ref app_path) = conventions.app_path {
        output.push_str(&format!(
            "#[path = \"{}\"]\nmod app;\n\n",
            app_path.display()
        ));
    }
    if let Some(ref mw_path) = conventions.middleware_path {
        output.push_str(&format!(
            "#[path = \"{}\"]\nmod middleware;\n\n",
            mw_path.display()
        ));
    }

    // Main function
    output.push_str(
        "#[allow(unused_mut, unused_variables, clippy::unit_arg, clippy::clone_on_copy)]\n",
    );
    output.push_str("#[tokio::main]\n");
    output.push_str("async fn main() -> anyhow::Result<()> {\n");
    output.push_str("    vega::server::init_tracing();\n\n");
    output.push_str("    let config_path = std::path::Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(\"Vega.toml\");\n");
    output.push_str("    let config = vega::config::VegaConfig::from_path(&config_path)?;\n");
    output.push_str("    let _manifest = vega::core::RouteManifest {\n");
    output.push_str("        pages: vega_generated_page_routes(),\n");
    output.push_str("        api: vega_generated_api_routes(),\n");
    output.push_str("    };\n\n");

    // Create state
    if conventions.has_app {
        output.push_str("    let state = app::create_state(_manifest, &config).await?;\n\n");
    } else {
        output.push_str("    let state = ();\n\n");
    }

    // Build router
    let state_type = conventions.state_type();
    output.push_str(&format!(
        "    let mut router: vega::server::web::Router<{}> = vega::server::web::Router::new();\n",
        state_type
    ));
    output.push_str("    router = vega_register_page_runtime_routes(router, state.clone());\n");
    output.push_str("    router = vega_register_api_runtime_routes(router, state.clone());\n\n");

    // Auto-register POST handlers detected in pages
    if !conventions.page_post_handlers.is_empty() {
        output.push_str("    // Auto-detected POST handlers\n");

        let has_admin_mw = conventions
            .middleware_fns
            .contains(&"require_admin_page".to_string());
        if has_admin_mw {
            output.push_str("    let admin_post_mw = vega::server::web::from_fn_with_state(state.clone(), crate::middleware::require_admin_page);\n");
        }

        for (route_path, module_path) in &conventions.page_post_handlers {
            let axum_path = to_axum_path(route_path);
            let mut post_route = format!(
                "vega::server::web::post(crate::{}::post_handler)",
                module_path
            );
            // Apply admin middleware if route starts with /admin
            if has_admin_mw && route_path.starts_with("/admin") {
                post_route.push_str(".layer(admin_post_mw.clone())");
            }
            output.push_str(&format!(
                "    router = router.route(\"{}\", {});\n",
                axum_path, post_route
            ));
        }
        output.push('\n');
    }

    // Static file serving
    output.push_str("    let public_dir = std::path::Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(\"public\");\n");
    output.push_str("    let app = router\n");

    if conventions.has_app {
        output.push_str("        .with_state::<()>(state)\n");
    }

    output.push_str("        .fallback_service(vega::server::static_file_service(&public_dir))\n");
    output.push_str("        .layer(vega::middleware::logger_layer())\n");
    output.push_str("        .layer(vega::middleware::cors_layer())\n");
    output.push_str("        .layer(vega::middleware::security_headers_layer());\n\n");

    output.push_str(
        "    vega::server::run_with_shutdown(app, &config.server.host, config.server.port).await\n",
    );
    output.push_str("}\n");

    fs::write(out_file, output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_segments() {
        assert_eq!(parse_segment("index"), SegmentKind::Index);
        assert_eq!(
            parse_segment("[slug]"),
            SegmentKind::Dynamic("slug".to_string())
        );
        assert_eq!(
            parse_segment("[...path]"),
            SegmentKind::CatchAll("path".to_string())
        );
        assert_eq!(
            parse_segment("(auth)"),
            SegmentKind::Group("auth".to_string())
        );
        assert_eq!(sanitize_module_name("[slug]"), "slug_dynamic");
        assert_eq!(sanitize_module_name("(auth)"), "auth_group");
    }

    #[test]
    fn scan_layout_chain() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let pages = tmp.path().join("pages");
        fs::create_dir_all(pages.join("blog")).expect("mkdir");

        fs::write(pages.join("_layout.rs"), "pub fn Layout() {}\n").expect("write");
        fs::write(pages.join("blog/_layout.rs"), "pub fn Layout() {}\n").expect("write");
        fs::write(pages.join("blog/[slug].rs"), "pub fn Page() {}\n").expect("write");

        let entries = scan_pages(&pages).expect("scan pages");
        let blog_post = entries
            .iter()
            .find(|entry| entry.route_path == "/blog/:slug")
            .expect("blog post route");

        assert_eq!(
            blog_post.layouts,
            vec![
                "pages::_layout".to_string(),
                "pages::blog::_layout".to_string()
            ]
        );
    }

    #[test]
    fn infer_api_macros() {
        let tmp = tempfile::NamedTempFile::new().expect("tmpfile");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(tmp.path())
            .expect("open");
        write!(
            file,
            "#[get]\nfn a() {{}}\n#[vega::post]\nfn b() {{}}\n#[vega::delete]\nfn c() {{}}\n"
        )
        .expect("write");

        let methods = infer_api_methods(tmp.path()).expect("methods");
        assert!(methods.contains(&HttpMethod::Get));
        assert!(methods.contains(&HttpMethod::Post));
        assert!(methods.contains(&HttpMethod::Delete));
    }
}
