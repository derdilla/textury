mod login;
mod project;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .nest("/project/{id}", project::router())
        .nest("/login", login::router())
}
