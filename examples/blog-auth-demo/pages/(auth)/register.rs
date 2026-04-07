use crate::app::AppState;
use vega::server::{
    web::{Form, Html, IntoResponse, Redirect, Request, Response, State},
    AuthService, SessionStore,
};

#[vega::page(mode = "ssr")]
pub fn RegisterPage() -> &'static str {
    "Register"
}

#[derive(serde::Deserialize)]
pub struct RegisterForm {
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
            <h1>Create Account</h1>
            {error}
            <form method="post" action="/register">
                <div class="form-group">
                    <label for="email">Email</label>
                    <input type="email" id="email" name="email" placeholder="you@example.com" required />
                </div>
                <div class="form-group">
                    <label for="password">Password</label>
                    <input type="password" id="password" name="password" placeholder="••••••••" minlength="6" required />
                </div>
                <button type="submit" class="btn btn-primary" style="width:100%">Create Account</button>
            </form>
            <p style="text-align:center;margin-top:1rem;font-size:0.875rem;color:var(--ink-muted)">
                Already have an account? <a href="/login">Sign in</a>
            </p>
        </div>"#,
        error = error_html
    )
}

pub async fn handler(State(state): State<AppState>, req: Request) -> Html<String> {
    let user = crate::app::current_user_from_headers(req.headers(), &state).await;
    let body = render(None);
    Html(crate::pages::_layout::render_layout(
        "Register",
        &body,
        user.as_ref(),
        None,
    ))
}

pub async fn post_handler(
    State(state): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> Response {
    if form.password.len() < 6 {
        let body = render(Some("Password must be at least 6 characters."));
        return Html(crate::pages::_layout::render_layout(
            "Register",
            &body,
            None,
            None,
        ))
        .into_response();
    }

    match state.auth.register(&form.email, &form.password, "member").await {
        Ok(user) => {
            let token = state
                .sessions
                .create_session(user)
                .await
                .expect("session create");
            let cookie = vega::server::make_session_cookie(&token, state.secure_cookie, 86400);
            let mut resp = Redirect::to("/dashboard").into_response();
            resp.headers_mut().insert(
                axum::http::header::SET_COOKIE,
                cookie.parse().expect("cookie header"),
            );
            resp
        }
        Err(err) => {
            let body = render(Some(&err.to_string()));
            Html(crate::pages::_layout::render_layout(
                "Register",
                &body,
                None,
                None,
            ))
            .into_response()
        }
    }
}
