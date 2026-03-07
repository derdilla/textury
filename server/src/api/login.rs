use crate::api::{CreateAccountResult, App};
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tower_cookies::{Cookie, Cookies, Key};

pub const USER_COOKIE_NAME: &str = "user_private";
pub static KEY: OnceLock<Key> = OnceLock::new();

pub fn router(router: Router<App>) -> Router<App> {
    use axum::routing::{get, post};

    KEY.set(Key::generate()).unwrap();

    router
        .route("/login", get(username).post(login))
        .route("/register", post(register))
}

async fn register(State(app): State<App>, cookies: Cookies, Json(req): Json<CreateAccountRequest>) -> impl IntoResponse {
    match app.register(req.registration_token, req.username, req.email, req.password).await {
        CreateAccountResult::Created { id } => {
            cookies.private(KEY.get().unwrap())
                .add(Cookie::new(USER_COOKIE_NAME, id.to_string()));
            (StatusCode::OK, "success")
        }
        CreateAccountResult::BadRequest => (StatusCode::BAD_REQUEST, "Bad request contents (e.g. name or password too short)"),
        CreateAccountResult::Exists => (StatusCode::BAD_REQUEST, "There is a user with this email"),
        CreateAccountResult::ExpiredToken => (StatusCode::UNAUTHORIZED, "Registration token is expired"),
        CreateAccountResult::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid registration token"),
    }
}

async fn login(State(app): State<App>, cookies: Cookies, Json(req): Json<LoginRequest>) -> impl IntoResponse {
    let Some(user) = app.login(req.email, req.password).await else {
        return (StatusCode::UNAUTHORIZED, "Invalid email or password").into_response();
    };
    cookies.private(KEY.get().unwrap())
        .add(Cookie::new(USER_COOKIE_NAME, user.id.to_string()));
    (StatusCode::OK, Json(PrivateAccountInfo { username: user.name })).into_response()
}

#[axum::debug_handler]
async fn username(State(app): State<App>, cookies: Cookies) -> impl IntoResponse {
    let Some(user) = app.user(cookies).await else {
        return (StatusCode::NOT_FOUND, "logged out").into_response();
    };
    (StatusCode::OK, Json(PrivateAccountInfo { username: user.name })).into_response()
}

#[derive(Deserialize)]
struct CreateAccountRequest {
    username: String,
    email: String,
    password: String,
    registration_token: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct PrivateAccountInfo {
    username: String,
}
