use axum::{Router, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::Value;
use tower_sessions::Session;

pub fn router() -> Router {
    use axum::routing::{get, post};

    Router::new()
        .route("/", get(username))
        .route("/{username}", post(login))
}

async fn login(session: Session, Path(username): Path<String>) -> impl IntoResponse {
    session.clear().await;
    session
        .insert_value("username", Value::String(username))
        .await
        .unwrap();
    session.save().await.unwrap();
    StatusCode::OK
    //session.
}

async fn username(session: Session) -> String {
    let Ok(Some(Value::String(name))) = session.get_value("username").await else {
        return "logged out".to_string();
    };
    name
}
