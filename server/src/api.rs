mod login;
mod project;

use axum::Router;
use tower_cookies::{Cookies, cookie};

use crate::{
    api::login::{KEY, USER_COOKIE_NAME},
    app::App,
    model,
};

pub async fn router() -> Router<App> {
    use axum::routing::{any, get, post, put};

    KEY.set(cookie::Key::generate()).unwrap();

    Router::new()
        .route("/login", get(login::whoami).post(login::login))
        .route("/register", post(login::register))
        .route("/invite", get(login::create_invite))
        .route("/projects", get(project::list_projects))
        .route("/projects/share/{id}", get(project::share_project))
        .route("/projects/join/{token}", get(project::accept_share))
        .route(
            "/project/{id}",
            put(project::create_project).delete(project::delete_project),
        )
        .route("/project/{id}/edit/{*path}", any(project::yrs))
        .route(
            "/project/{id}/file/{*path}",
            put(project::add_or_update_file)
                .delete(project::delete_file)
                .get(project::get_file),
        )
        .route("/project/{id}/move/{*path}", post(project::move_file))
        .route("/project/{id}/index", get(project::get_index))
        .route("/project/{id}/last_compiled", get(project::last_compiled))
        .route("/project/{id}/meta", post(project::update_user_meta))
        .route("/project/{id}/compile/{*path}", post(project::compile))
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
