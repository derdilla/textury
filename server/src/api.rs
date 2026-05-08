mod login;
mod project;

use axum::{
    Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use tower_cookies::{Cookies, cookie};

use crate::{
    api::login::{KEY, USER_COOKIE_NAME},
    app::App,
    model,
};

pub async fn router(app: App) -> Router<App> {
    use axum::routing::{any, get, post, put};

    KEY.set(cookie::Key::generate()).unwrap();

    let authenticated_routes = Router::new()
        .route("/invite", get(login::create_invite))
        .route("/projects", get(project::list_projects))
        .route("/projects/share/{id}", get(project::share_project))
        .route("/projects/join/{token}", get(project::accept_share))
        .route(
            "/project/{id}",
            put(project::create_project).delete(project::delete_project),
        )
        .route("/project/edit/{id}", any(project::edit_ws))
        .route("/project/{id}/index", get(project::get_index))
        .route("/project/{id}/last_compiled", get(project::last_compiled))
        .route("/project/{id}/meta", post(project::update_user_meta))
        .route("/project/{id}/zip", get(project::zip))
        // Layer must be last:
        .route_layer(middleware::from_fn_with_state(app, user_middleware));

    Router::new()
        .route("/login", get(login::whoami).post(login::login))
        .route("/register", post(login::register))
        .merge(authenticated_routes)
}

/// Gets user if in DB and id is in cookie.
pub async fn user(app: &App, cookies: Cookies) -> Option<model::user::Model> {
    let key = KEY.get().unwrap();
    let private_cookies = cookies.private(key);
    let Some(user_cookie) = private_cookies.get(USER_COOKIE_NAME) else {
        return None;
    };
    let user_id = user_cookie.value().parse::<i32>().unwrap();
    app.user_from_id(user_id).await
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
