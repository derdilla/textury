use axum::{Extension, Json, extract::{Path, State}, http::StatusCode, response::{IntoResponse, Response}};

use crate::{api, app::{self, App}};

pub(crate) async fn get_settings(
    State(app): State<App>,
) -> Json<api::types::ServerSettings> {
    let settings = app.settings();
    Json(api::types::ServerSettings {
        open_sign_ups: settings.open_sign_ups(),
        user_projects: settings.user_projects(),
        open_sharing: settings.open_sharing(),
    })
}

pub(crate) async fn set_open_sign_ups(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Json(value): Json<bool>,
) -> Result<(), String> {
    assert!(user.is_admin);
    app.settings().set_open_sign_ups(value).await
}

pub(crate) async fn set_user_projects(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Json(value): Json<bool>,
) -> Result<(), String> {
    assert!(user.is_admin);
    app.settings().set_user_projects(value).await
}

pub(crate) async fn set_open_sharing(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Json(value): Json<bool>,
) -> Result<(), String> {
    assert!(user.is_admin);
    app.settings().set_open_sharing(value).await
}

pub(crate) async fn list_users(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
) -> Json<Vec<api::types::User>> {
    assert!(user.is_admin);
    Json(app.all_users().await.into_iter()
        .map(|user| api::types::User { id: user.id.into(), username: user.name })
        .collect())
}

pub(crate) async fn get_user_details(
    State(app): State<App>,
    Extension(requesting_user): Extension<app::UserPrivateSummary>,
    Path(user_id): Path<api::types::UserId>,
) -> Response {
    assert!(requesting_user.is_admin);
    let user_id = user_id.into();
    let Some(user) = app.user_from_id(user_id).await else {
        return StatusCode::NOT_FOUND.into_response()
    };
    let Some(projects) = app.list_projects(user_id).await else {
        return StatusCode::NOT_FOUND.into_response()
    };
    let projects = api::project::compile_projects_response(app, user.id, projects).await;
    Json(api::types::AdminUserInfo {
        id: user.id.into(),
        email: user.email,
        username: user.name,
        is_admin: user.is_admin,
        projects,
    }).into_response()
}

pub(crate) async fn delete_user(
    State(app): State<App>,
    Extension(requesting_user): Extension<app::UserPrivateSummary>,
    Path(user_id): Path<api::types::UserId>,
) -> (StatusCode, &'static str) {
    assert!(requesting_user.is_admin);
    let user_id = user_id.into();
    if !app.can_change(requesting_user.id, user_id).await {
        return (StatusCode::UNAUTHORIZED, "Not allowed to modify this user")
    }

    app.delete_user(user_id).await;

    (StatusCode::OK, "User deleted successfully")
}

pub(crate) async fn set_user_username(
    State(app): State<App>,
    Extension(requesting_user): Extension<app::UserPrivateSummary>,
    Path(user_id): Path<api::types::UserId>,
    Json(new_name): Json<String>,
) -> (StatusCode, &'static str) {
    assert!(requesting_user.is_admin);
    let user_id = user_id.into();
    if !app.can_change(requesting_user.id, user_id).await {
        return (StatusCode::UNAUTHORIZED, "Not allowed to modify another admin user")
    }
    app.set_username(user_id, new_name).await;
    (StatusCode::OK, "User changed successfully")
}

pub(crate) async fn set_user_email(
    State(app): State<App>,
    Extension(requesting_user): Extension<app::UserPrivateSummary>,
    Path(user_id): Path<api::types::UserId>,
    Json(new_email): Json<String>,
) -> (StatusCode, &'static str) {
    assert!(requesting_user.is_admin);
    let user_id = user_id.into();
    if !app.can_change(requesting_user.id, user_id).await {
        return (StatusCode::UNAUTHORIZED, "Not allowed to modify another admin user")
    }

    if !app.set_email(user_id, new_email).await {
        return (StatusCode::CONFLICT, "There is already another user with that email");
    }
    (StatusCode::OK, "User changed successfully")
}

pub(crate) async fn set_user_is_admin(
    State(app): State<App>,
    Extension(requesting_user): Extension<app::UserPrivateSummary>,
    Path(user_id): Path<api::types::UserId>,
    Json(is_admin): Json<bool>,
) -> (StatusCode, &'static str) {
    assert!(requesting_user.is_admin);
    let user_id = user_id.into();
    if !app.can_change(requesting_user.id, user_id).await {
        return (StatusCode::UNAUTHORIZED, "Not allowed to modify another admin user")
    }

    app.set_is_admin(user_id, requesting_user.id, is_admin).await;
    (StatusCode::OK, "User changed successfully")
}
