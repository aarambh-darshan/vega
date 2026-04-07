#[vega::layout]
pub fn Layout(children: fn() -> &'static str) -> &'static str {
    let _ = children;
    "Root Layout"
}

pub fn render_layout(
    title: &str,
    body: &str,
    user: Option<&vega::server::AuthUser>,
    flash: Option<&str>,
) -> String {
    let user_links = match user {
        Some(user) => format!(
            r#"<div class="user-info">
                <span class="user-badge">{role}</span>
                <span>{email}</span>
                <a href="/dashboard" class="nav-link">Dashboard</a>
                {admin_link}
                <form method="post" action="/logout" style="display:inline">
                    <button type="submit" class="btn btn-outline btn-sm">Logout</button>
                </form>
            </div>"#,
            email = vega::core::html_escape(&user.email),
            role = vega::core::html_escape(&user.role),
            admin_link = if user.role == "admin" {
                r#"<a href="/admin" class="nav-link">Admin</a>"#
            } else {
                ""
            },
        ),
        None => r#"<div class="auth-links">
                <a href="/login" class="btn btn-outline btn-sm">Sign In</a>
                <a href="/register" class="btn btn-primary btn-sm">Get Started</a>
            </div>"#.to_string(),
    };

    let flash_html = match flash {
        Some(msg) => format!(
            r#"<div class="flash">{}</div>"#,
            vega::core::html_escape(msg)
        ),
        None => String::new(),
    };

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{title} — Vega</title>
    <meta name="description" content="Vega web framework demo — file-based routing, SSR, auth, and API routes for Rust" />
    <link rel="icon" href="/favicon.ico" />
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet" />
    <style>
        :root {{
            --bg: #fafaf9;
            --bg-alt: #f5f5f4;
            --surface: #ffffff;
            --ink: #1c1917;
            --ink-muted: #78716c;
            --accent: #0d9488;
            --accent-hover: #0f766e;
            --accent-light: #ccfbf1;
            --danger: #dc2626;
            --danger-light: #fef2f2;
            --line: #e7e5e4;
            --radius: 10px;
            --radius-lg: 16px;
            --shadow-sm: 0 1px 2px rgba(0,0,0,0.05);
            --shadow: 0 1px 3px rgba(0,0,0,0.08), 0 1px 2px rgba(0,0,0,0.06);
            --shadow-lg: 0 10px 15px -3px rgba(0,0,0,0.08), 0 4px 6px rgba(0,0,0,0.04);
            --transition: 150ms cubic-bezier(0.4, 0, 0.2, 1);
        }}

        * {{ box-sizing: border-box; margin: 0; padding: 0; }}

        body {{
            font-family: 'Inter', system-ui, -apple-system, sans-serif;
            background: var(--bg);
            color: var(--ink);
            line-height: 1.6;
            -webkit-font-smoothing: antialiased;
        }}

        /* Header */
        header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 0.75rem 1.5rem;
            border-bottom: 1px solid var(--line);
            background: rgba(255,255,255,0.92);
            backdrop-filter: blur(12px);
            -webkit-backdrop-filter: blur(12px);
            position: sticky;
            top: 0;
            z-index: 100;
        }}

        .logo {{
            font-weight: 700;
            font-size: 1.25rem;
            color: var(--accent);
            text-decoration: none;
            letter-spacing: -0.02em;
        }}

        .logo span {{ color: var(--ink); }}

        nav {{ display: flex; align-items: center; gap: 0.25rem; }}

        .nav-link {{
            padding: 0.4rem 0.75rem;
            color: var(--ink-muted);
            text-decoration: none;
            font-size: 0.875rem;
            font-weight: 500;
            border-radius: var(--radius);
            transition: all var(--transition);
        }}

        .nav-link:hover {{ color: var(--ink); background: var(--bg-alt); }}
        .nav-link.active {{ color: var(--accent); background: var(--accent-light); }}

        .user-info {{ display: flex; align-items: center; gap: 0.75rem; font-size: 0.875rem; }}
        .auth-links {{ display: flex; gap: 0.5rem; }}

        .user-badge {{
            background: var(--accent-light);
            color: var(--accent);
            padding: 0.15rem 0.5rem;
            border-radius: 999px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}

        /* Main */
        main {{
            max-width: 960px;
            margin: 0 auto;
            padding: 2rem 1.5rem;
            min-height: calc(100vh - 200px);
        }}

        /* Typography */
        h1 {{ font-size: 2rem; font-weight: 700; letter-spacing: -0.03em; margin-bottom: 0.75rem; color: var(--ink); }}
        h2 {{ font-size: 1.5rem; font-weight: 600; letter-spacing: -0.02em; margin-bottom: 0.5rem; margin-top: 2rem; }}
        h3 {{ font-size: 1.125rem; font-weight: 600; margin-bottom: 0.375rem; }}
        p {{ color: var(--ink-muted); margin-bottom: 1rem; }}
        a {{ color: var(--accent); text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}

        /* Buttons */
        .btn {{
            display: inline-flex;
            align-items: center;
            gap: 0.375rem;
            padding: 0.5rem 1rem;
            border: 1px solid transparent;
            border-radius: var(--radius);
            font-size: 0.875rem;
            font-weight: 500;
            text-decoration: none;
            cursor: pointer;
            transition: all var(--transition);
            line-height: 1.4;
            font-family: inherit;
        }}

        .btn-primary {{
            background: var(--accent);
            color: #fff;
            border-color: var(--accent);
        }}

        .btn-primary:hover {{ background: var(--accent-hover); text-decoration: none; }}

        .btn-outline {{
            background: transparent;
            color: var(--ink);
            border-color: var(--line);
        }}

        .btn-outline:hover {{ background: var(--bg-alt); border-color: var(--ink-muted); text-decoration: none; }}

        .btn-danger {{
            background: var(--danger);
            color: #fff;
            border-color: var(--danger);
        }}

        .btn-danger:hover {{ opacity: 0.9; text-decoration: none; }}

        .btn-sm {{ padding: 0.3rem 0.65rem; font-size: 0.8rem; }}

        /* Cards */
        .card {{
            background: var(--surface);
            border: 1px solid var(--line);
            border-radius: var(--radius-lg);
            padding: 1.25rem;
            margin-bottom: 1rem;
            box-shadow: var(--shadow-sm);
            transition: box-shadow var(--transition), transform var(--transition);
        }}

        .card:hover {{ box-shadow: var(--shadow); }}
        .card h3 a {{ color: var(--ink); }}
        .card h3 a:hover {{ color: var(--accent); text-decoration: none; }}

        /* Tags */
        .tag {{
            display: inline-block;
            background: var(--bg-alt);
            color: var(--ink-muted);
            padding: 0.15rem 0.5rem;
            border-radius: 999px;
            font-size: 0.75rem;
            font-weight: 500;
            margin-right: 0.25rem;
        }}

        /* Forms */
        .form-group {{ margin-bottom: 1.25rem; }}

        .form-group label {{
            display: block;
            font-size: 0.875rem;
            font-weight: 500;
            margin-bottom: 0.375rem;
            color: var(--ink);
        }}

        input[type="text"], input[type="email"], input[type="password"], textarea, select {{
            width: 100%;
            padding: 0.55rem 0.75rem;
            border: 1px solid var(--line);
            border-radius: var(--radius);
            font-size: 0.875rem;
            font-family: inherit;
            transition: border-color var(--transition), box-shadow var(--transition);
            background: var(--surface);
        }}

        input:focus, textarea:focus, select:focus {{
            outline: none;
            border-color: var(--accent);
            box-shadow: 0 0 0 3px var(--accent-light);
        }}

        textarea {{ min-height: 120px; resize: vertical; }}

        /* Flash messages */
        .flash {{
            background: var(--accent-light);
            color: var(--accent-hover);
            padding: 0.75rem 1rem;
            border-radius: var(--radius);
            margin-bottom: 1.5rem;
            font-size: 0.875rem;
            font-weight: 500;
            border-left: 3px solid var(--accent);
        }}

        .flash-error {{
            background: var(--danger-light);
            color: var(--danger);
            border-left-color: var(--danger);
        }}

        /* Error pages */
        .error-page {{ text-align: center; padding: 4rem 1rem; }}
        .error-page h1 {{ font-size: 4rem; color: var(--ink-muted); margin-bottom: 0.5rem; }}
        .error-page p {{ font-size: 1.125rem; margin-bottom: 2rem; }}

        /* Code */
        code {{
            background: var(--bg-alt);
            padding: 0.15rem 0.4rem;
            border-radius: 6px;
            font-size: 0.85em;
            font-family: 'SF Mono', Consolas, monospace;
        }}

        pre {{
            background: #1e293b;
            color: #e2e8f0;
            padding: 1rem 1.25rem;
            border-radius: var(--radius-lg);
            overflow-x: auto;
            font-size: 0.85rem;
            line-height: 1.7;
            margin: 1rem 0;
        }}

        pre code {{ background: none; padding: 0; color: inherit; }}

        /* Grid */
        .grid {{ display: grid; gap: 1rem; }}
        .grid-2 {{ grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); }}
        .grid-3 {{ grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); }}

        /* Table */
        table {{ width: 100%; border-collapse: collapse; margin: 1rem 0; }}
        th, td {{ padding: 0.6rem 0.75rem; text-align: left; border-bottom: 1px solid var(--line); font-size: 0.875rem; }}
        th {{ font-weight: 600; color: var(--ink-muted); font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.05em; }}
        tr:hover td {{ background: var(--bg-alt); }}

        /* Stats */
        .stat {{ text-align: center; padding: 1.25rem; }}
        .stat-value {{ font-size: 2rem; font-weight: 700; color: var(--accent); }}
        .stat-label {{ font-size: 0.8rem; color: var(--ink-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-top: 0.25rem; }}

        /* Hero */
        .hero {{
            text-align: center;
            padding: 4rem 1rem 3rem;
        }}

        .hero h1 {{
            font-size: 2.75rem;
            letter-spacing: -0.04em;
            line-height: 1.15;
            margin-bottom: 1rem;
        }}

        .hero h1 span {{ color: var(--accent); }}
        .hero p {{ font-size: 1.125rem; max-width: 600px; margin: 0 auto 2rem; }}

        /* Section */
        .section {{ padding: 3rem 0; }}
        .section-title {{ text-align: center; margin-bottom: 2rem; }}

        /* Pagination */
        .pagination {{ display: flex; gap: 0.375rem; align-items: center; margin-top: 1.5rem; }}
        .pagination a, .pagination span {{ padding: 0.35rem 0.65rem; border-radius: var(--radius); font-size: 0.875rem; font-weight: 500; }}
        .pagination a {{ border: 1px solid var(--line); color: var(--ink-muted); text-decoration: none; }}
        .pagination a:hover {{ background: var(--bg-alt); }}
        .pagination .current {{ background: var(--accent); color: #fff; }}

        /* Footer */
        footer {{
            border-top: 1px solid var(--line);
            margin-top: 3rem;
            padding: 1.5rem;
            text-align: center;
            color: var(--ink-muted);
            font-size: 0.8rem;
        }}

        footer a {{ color: var(--accent); }}

        /* Auth form wrapper */
        .auth-form {{
            max-width: 400px;
            margin: 2rem auto;
            padding: 2rem;
            background: var(--surface);
            border: 1px solid var(--line);
            border-radius: var(--radius-lg);
            box-shadow: var(--shadow-lg);
        }}

        .auth-form h1 {{ text-align: center; margin-bottom: 1.5rem; }}

        /* Responsive */
        @media (max-width: 640px) {{
            header {{ flex-direction: column; gap: 0.75rem; align-items: stretch; }}
            nav {{ justify-content: center; flex-wrap: wrap; }}
            .hero h1 {{ font-size: 2rem; }}
            main {{ padding: 1.25rem 1rem; }}
            .grid-2, .grid-3 {{ grid-template-columns: 1fr; }}
            .user-info {{ flex-wrap: wrap; justify-content: center; }}
        }}
    </style>
</head>
<body>
    <header>
        <div style="display:flex;align-items:center;gap:1.5rem">
            <a href="/" class="logo">वेग <span>Vega</span></a>
            <nav>
                <a href="/" class="nav-link">Home</a>
                <a href="/blog" class="nav-link">Blog</a>
                <a href="/docs" class="nav-link">Docs</a>
                <a href="/about" class="nav-link">About</a>
            </nav>
        </div>
        {user_links}
    </header>
    {flash}
    <main>{body}</main>
    <footer>
        <p>Built with <a href="https://github.com/AarambhDevHub/vega">Vega</a> — a Next.js-inspired web framework for Rust</p>
        <p style="margin-top:0.25rem">वेग (speed, momentum) · v0.9.0</p>
    </footer>
</body>
</html>"#,
        title = vega::core::html_escape(title),
        user_links = user_links,
        flash = flash_html,
        body = body,
    )
}
