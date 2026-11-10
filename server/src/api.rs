mod login;
mod project;
mod settings;
pub mod types;
pub mod websocket;

use axum::{
    Router, extract::{Request, State}, http::StatusCode, middleware::{self, Next}, response::{IntoResponse, Response}, routing::delete,
};
use tower_cookies::{Cookies, cookie};

use crate::{
    api::{login::{KEY, USER_COOKIE_NAME}, types::UserId}, app::{self, App}
};

pub async fn router(app: App) -> Router<App> {
    use axum::routing::{any, get, post, put};

    KEY.set(cookie::Key::generate()).unwrap();

    let authenticated_routes = Router::new()
        .route("/user/username", put(login::set_username))
        .route("/user/email", put(login::set_email))
        .route("/user/clientSettings", put(login::set_client_settings))
        .route("/contacts", get(login::contacts))
        .route("/projects/index", get(project::list_projects))
        .route("/projects/preview", get(project::preview_share))
        .route("/projects/join", post(project::accept_share))
        .route("/projects/create", post(project::create_project))
        .route("/project/{id}", delete(project::delete_project))
        .route("/project/{id}/share", post(project::share_project))
        .route("/project/{id}/publicShare", post(project::share_project_publicly)
            .get(project::get_public_share_info).delete(project::unshare_publicly))
        .route("/project/{id}/edit", any(project::edit_ws))
        .route("/project/{id}/latestArtifact", get(project::last_compiled))
        .route("/project/{id}/meta", put(project::update_user_meta))
        .route("/project/{id}/zip", get(project::zip))
        .route("/project/{id}/title", put(project::set_title))
        .route("/project/{id}/leave", post(project::leave))
        .route("/project/{id}/members", get(project::members))
        .route("/project/{pid}/members/{uid}/kick", post(project::kick))
        .route("/project/{pid}/members/{uid}/transferOwnership", post(project::transfer_ownership))
        .route("/project/{pid}/members/{uid}/permissions", put(project::change_permissions))
        // Layer must be last:
        .route_layer(middleware::from_fn_with_state(app.clone(), user_middleware));

    let server_setting = Router::new()
        .route("/settings/openSignUps", put(settings::set_open_sign_ups))
        .route("/settings/userProjects", put(settings::set_user_projects))
        .route("/settings/openSharing", put(settings::set_open_sharing))
        .route("/settings/maxCompileDuration", put(settings::set_max_compile_duration))
        .route("/settings/maxCompileMemory", put(settings::set_max_compile_memory))
        .route("/settings/maxProjectCount", put(settings::set_max_project_count))
        .route("/settings/users", get(settings::list_users))
        .route("/settings/users/invite", post(login::create_invite))
        .route("/settings/users/{userId}",
            get(settings::get_user_details)
                .delete(settings::delete_user))
        .route("/settings/users/{userId}/username", put(settings::set_user_username))
        .route("/settings/users/{userId}/email", put(settings::set_user_email))
        .route("/settings/users/{userId}/isAdmin", put(settings::set_user_is_admin))
        .route_layer(middleware::from_fn_with_state(app, admin_middleware));

    Router::new()
        .route("/user", get(login::whoami).post(login::register).delete(login::delete_account))
        .route("/user/login", post(login::login))
        .route("/user/singleProjectUser", post(login::register_single_project_user))
        .route("/user/logout", post(login::logout))
        .route("/settings", get(settings::get_settings)) // public for open_sign_ups setting
        .merge(server_setting)
        .merge(authenticated_routes)
}

/// Gets user if in DB and id is in cookie.
pub async fn user(app: &App, cookies: &Cookies) -> Option<app::UserPrivateSummary> {
    let key = KEY.get().unwrap();
    let private_cookies = cookies.private(key);
    let user_cookie = private_cookies.get(USER_COOKIE_NAME)?;
    let user_id = UserId::try_from(user_cookie.value()).ok()?;
    app.user_from_id(user_id).await
}

async fn user_middleware(
    State(app): State<App>,
    cookies: Cookies,
    mut req: Request,
    next: Next,
) -> Response {
    let Some(user) = user(&app, &cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in").into_response();
    };

    req.extensions_mut().insert(user);

    next.run(req).await
}

async fn admin_middleware(
    State(app): State<App>,
    cookies: Cookies,
    mut req: Request,
    next: Next,
) -> Response {
    let Some(user) = user(&app, &cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in").into_response();
    };
    if !user.is_admin {
        return (StatusCode::UNAUTHORIZED, "Insufficient privileges: Shoo!").into_response();
    }

    req.extensions_mut().insert(user);

    next.run(req).await
}
