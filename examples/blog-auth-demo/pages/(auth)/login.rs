use crate::app::AppState;
use vega::server::{
    make_session_cookie,
    web::{Form, Html, IntoResponse, Redirect, Request, Response, State},
    AuthService, SessionStore,
};

#[vega::page(mode = "ssr")]
pub fn LoginPage() -> &'static str {
    "Login"
}

#[derive(serde::Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
}

pub fn render(error: Option<&str>) -> String {
    let error_html = match error {
        Some(msg) => format!(
            r#"<div class="flash flash-error">{}</div>"#,
            vega::core::html_escape(msg)
        ),
        None => String::new(),
    };

    format!(
        r#"<div class="auth-form">
            <h1>Sign In</h1>
            {error}
            <form method="post" action="/login">
                <div class="form-group">
                    <label for="email">Email</label>
                    <input type="email" id="email" name="email" placeholder="you@example.com" required />
                </div>
                <div class="form-group">
                    <label for="password">Password</label>
                    <input type="password" id="password" name="password" placeholder="••••••••" required />
                </div>
                <button type="submit" class="btn btn-primary" style="width:100%">Sign In</button>
            </form>
            <p style="text-align:center;margin-top:1rem;font-size:0.875rem;color:var(--ink-muted)">
                Don't have an account? <a href="/register">Register</a>
            </p>
            <div style="margin-top:1.5rem;padding-top:1rem;border-top:1px solid var(--line);font-size:0.8rem;color:var(--ink-muted)">
                <p><strong>Demo accounts:</strong></p>
                <p>Admin: admin@vega.dev / admin123</p>
                <p>Member: member@vega.dev / member123</p>
            </div>
        </div>"#,
        error = error_html
    )
}

pub async fn handler(State(state): State<AppState>, req: Request) -> Html<String> {
    let user = crate::app::current_user_from_headers(req.headers(), &state).await;
    let body = render(None);
    Html(crate::pages::_layout::render_layout(
        "Sign In",
        &body,
        user.as_ref(),
        None,
    ))
}

pub async fn post_handler(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Response {
    match state.auth.login(&form.email, &form.password).await {
        Ok(Some(user)) => {
            let token = state
                .sessions
                .create_session(user)
                .await
                .expect("session create");
            let cookie = make_session_cookie(&token, state.secure_cookie, 86400);
            let mut resp = Redirect::to("/dashboard").into_response();
            resp.headers_mut().insert(
                axum::http::header::SET_COOKIE,
                cookie.parse().expect("cookie header"),
            );
            resp
        }
        _ => {
            let body = render(Some("Invalid email or password. Please try again."));
            Html(crate::pages::_layout::render_layout(
                "Sign In",
                &body,
                None,
                None,
            ))
            .into_response()
        }
    }
}
