use crate::app::{self, App, CreateAccountResult};
use api_types::*;
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
    Json(req): Json<CreateAccountRequest>,
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
            (StatusCode::OK, "success")
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
            (StatusCode::UNAUTHORIZED, "Invalid registration token")
        }
    }
}

pub(crate) async fn login(
    State(app): State<App>,
    cookies: Cookies,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let Some(user) = app.login(req.email, req.password).await else {
        return (StatusCode::UNAUTHORIZED, "Invalid email or password").into_response();
    };
    cookies
        .private(KEY.get().unwrap())
        .add(Cookie::new(USER_COOKIE_NAME, user.id.to_string()));
    (
        StatusCode::OK,
        Json(PrivateAccountInfo {
            id: user.id.0,
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
        Json(PrivateAccountInfo {
            id: user.id.0,
            username: user.name,
            email: user.email,
            is_admin: user.is_admin,
        }),
    )
        .into_response()
}

pub async fn create_invite(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
) -> impl IntoResponse {
    use time::ext::*;

    let Some(registration_token) = app.create_invite(user).await else {
        return (
            StatusCode::UNAUTHORIZED,
            "You are not allowed to issue tokens",
        )
            .into_response();
    };

    Json(UserInvite {
        registration_token,
        expires: 15.minutes(),
    })
    .into_response()
}

pub(crate) mod api_types {
    use serde::{Deserialize, Serialize};
    use time::Duration;

    #[derive(Deserialize)]
    pub struct CreateAccountRequest {
        pub username: String,
        pub email: String,
        pub password: String,
        pub registration_token: String,
    }

    #[derive(Deserialize)]
    pub struct LoginRequest {
        pub email: String,
        pub password: String,
    }

    #[derive(Serialize)]
    pub struct PrivateAccountInfo {
        pub id: i64,
        pub username: String,
        pub email: String,
        pub is_admin: bool,
    }

    #[derive(Serialize)]
    pub struct UserInvite {
        pub registration_token: String,
        pub expires: Duration,
    }
}
