mod login;
mod project;
mod settings;
pub mod types;

use axum::{
    Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use tower_cookies::{Cookies, cookie};

use crate::{
    api::login::{KEY, USER_COOKIE_NAME}, app::{self, App, UserID}
};

pub async fn router(app: App) -> Router<App> {
    use axum::routing::{any, get, post, put};

    KEY.set(cookie::Key::generate()).unwrap();

    let authenticated_routes = Router::new()
        .route("/user/username", put(login::set_username))
        .route("/user/mail", put(login::set_email))
        .route("/user/clientSettings", put(login::set_client_settings))
        .route("/contacts", get(login::contacts))
        .route("/invite", post(login::create_invite))
        .route("/projects/index", get(project::list_projects))
        .route("/projects/join", get(project::accept_share))
        .route(
            "/project/{id}",
            put(project::create_project).delete(project::delete_project),
        )
        .route("/project/{id}/share", get(project::share_project))
        .route("/project/{id}/edit", any(project::edit_ws))
        .route("/project/{id}/latestArtifact", get(project::last_compiled))
        .route("/project/{id}/meta", put(project::update_user_meta))
        .route("/project/{id}/zip", get(project::zip))
        .route("/project/{id}/title", put(project::set_title))
        .route("/project/{id}/leave", post(project::leave))
        // Layer must be last:
        .route_layer(middleware::from_fn_with_state(app.clone(), user_middleware));

    let server_setting = Router::new()
        .route("/settings", get(settings::get_settings))
        .route("/settings/openSignUps", put(settings::set_open_sign_ups))
        .route("/settings/userProjects", put(settings::set_user_projects))
        .route("/settings/openSharing", put(settings::set_open_sharing))
        .route_layer(middleware::from_fn_with_state(app, admin_middleware));

    Router::new()
        .route("/user", get(login::whoami).post(login::register))
        .route("/user/login", post(login::login))
        .route("/user/logout", post(login::logout))
        .merge(server_setting)
        .merge(authenticated_routes)
}

/// Gets user if in DB and id is in cookie.
pub async fn user(app: &App, cookies: Cookies) -> Option<app::UserPrivateSummary> {
    let key = KEY.get().unwrap();
    let private_cookies = cookies.private(key);
    let user_cookie = private_cookies.get(USER_COOKIE_NAME)?;
    let user_id = user_cookie.value().parse::<i64>().unwrap();
    app.user_from_id(UserID(user_id)).await
}

async fn user_middleware(
    State(app): State<App>,
    cookies: Cookies,
    mut req: Request,
    next: Next,
) -> Response {
    let Some(user) = user(&app, cookies).await else {
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
    let Some(user) = user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in").into_response();
    };
    if !user.is_admin {
        return (StatusCode::UNAUTHORIZED, "Insufficient privileges: Shoo!").into_response();
    }

    req.extensions_mut().insert(user);

    next.run(req).await
}

// TODO: test middleware guards permissions
