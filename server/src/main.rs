mod api;
pub mod model;
mod users;
mod workers;

use crate::api::App;
use axum::Router;
use axum::response::Html;
use axum::routing::get;
use sea_orm::Database;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;

const FRONTEND: &str = "/var/home/rr/Documents/dillaleaf/frontend/dist";

#[tokio::main]
async fn main() {
    // TODO: remove dbg code, proper location, setup
    let db = Database::connect("sqlite::memory:").await.unwrap();

    assert!(db.ping().await.is_ok());
    let state = App::new(db).await;
    println!("Initial token: {}", state.add_registration_token().await);

    let app = Router::new()
        .nest("/api/v1", api::router().await)
        .fallback_service(ServiceBuilder::new().service(ServeDir::new(FRONTEND)));
    let app = app.with_state(state).layer(CookieManagerLayer::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    /*let mut worker_manager = WorkerManager::default();
    let w = worker_manager
        .get_worker("https://codeberg.org/derdilla/AOC25.git".into())
        .await;

     */
}

// todo persistence

// Idea: use a CRDT for files. External services like DB and git sync act like any other client
//
// server manages:
// - users
// - prpjects
// - workers (The things on which the code compiles)
// -
