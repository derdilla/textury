use crate::api::types;
use crate::app::{self, App, CreateAccountResult};
use crate::token_manager;
use axum::Extension;
use axum::extract::State;
use axum::{Json, http::StatusCode, response::IntoResponse};
use std::sync::OnceLock;
use tower_cookies::{Cookie, Cookies, Key};
use tracing::warn;

pub const USER_COOKIE_NAME: &str = "user_private";
pub static KEY: OnceLock<Key> = OnceLock::new();

pub(crate) async fn register(
    State(app): State<App>,
    cookies: Cookies,
    Json(req): Json<types::CreateAccountRequest>,
) -> impl IntoResponse {
    match app
        .register(
            req.registration_token,
            req.username,
            req.email,
            req.password,
        )
        .await
    {
        CreateAccountResult::Created { id } => {
            cookies
                .private(KEY.get().unwrap())
                .add(Cookie::new(USER_COOKIE_NAME, id.to_string()));
            (StatusCode::OK, "Created and logged in")
        }
        CreateAccountResult::BadRequest => (
            StatusCode::BAD_REQUEST,
            "Bad request contents (e.g. name or password too short)",
        ),
        CreateAccountResult::Exists => (StatusCode::BAD_REQUEST, "There is a user with this email"),
        CreateAccountResult::ExpiredToken => {
            (StatusCode::UNAUTHORIZED, "Registration token is expired")
        }
        CreateAccountResult::InvalidToken => {
            (StatusCode::UNAUTHORIZED, "Registration token not found")
        }
    }
}

pub(crate) async fn login(
    State(app): State<App>,
    cookies: Cookies,
    Json(req): Json<types::LoginRequest>,
) -> impl IntoResponse {
    let Some(user) = app.login(req.email, req.password).await else {
        return (StatusCode::UNAUTHORIZED, "Invalid email or password").into_response();
    };
    cookies
        .private(KEY.get().unwrap())
        .add(Cookie::new(USER_COOKIE_NAME, user.id.to_string()));
    (
        StatusCode::OK,
        Json(types::PrivateAccountInfo {
            id: user.id.into(),
            username: user.name,
            email: user.email,
            is_admin: user.is_admin,
        }),
    )
        .into_response()
}

pub(crate) async fn logout(cookies: Cookies) -> impl IntoResponse {
    let cookies = cookies.private(KEY.get().unwrap());
    let Some(old_cookie) = cookies.get(USER_COOKIE_NAME) else {
        warn!("Logout called - didn't find a cookie");
        return StatusCode::UNAUTHORIZED;
    };
    cookies.remove(old_cookie);
    StatusCode::OK
}

pub(crate) async fn whoami(State(app): State<App>, cookies: Cookies) -> impl IntoResponse {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in").into_response();
    };
    (
        StatusCode::OK,
        Json(types::PrivateAccountInfo {
            id: user.id.into(),
            username: user.name,
            email: user.email,
            is_admin: user.is_admin,
        }),
    )
        .into_response()
}

pub async fn contacts(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
) -> Json<types::ContactsList> {
    let contacts = app.contacts(user.id).await.into_iter()
        .map(|user| types::User { id: user.id.into(), username: user.name });
    Json(types::ContactsList(contacts.collect()))
}

pub async fn create_invite(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
) -> impl IntoResponse {
    let Some(registration_token) = app.create_invite(user).await else {
        return (
            StatusCode::UNAUTHORIZED,
            "You are not allowed to issue tokens",
        )
            .into_response();
    };

    Json(types::UserInvite {
        registration_token,
        expires: token_manager::DEFAULT_REGISTRATION_EXPIRERY.whole_minutes(),
    })
    .into_response()
}
