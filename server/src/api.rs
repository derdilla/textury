mod login;
mod project;

use axum::Router;
use tower_cookies::Cookies;

use crate::{
    api::login::{KEY, USER_COOKIE_NAME},
    app::App,
    model,
};

pub async fn router() -> Router<App> {
    use axum::routing::{get, post, put};

    Router::new()
        .route("/login", get(login::whoami).post(login::login))
        .route("/register", post(login::register))
        .route("/projects", get(project::list_projects))
        .route(
            "/project/{id}",
            /*get(get_project)*/ put(project::create_project),
        )
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
