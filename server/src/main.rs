mod api;
mod app;
mod model;
mod users;
mod workers;

use crate::app::App;
use axum::{Router, response::Html, routing::get};
use sea_orm::Database;
use std::{net::SocketAddr, str::FromStr};
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;

const FRONTEND: &str = "/var/home/rr/Documents/dillaleaf/frontend/dist";
const DB_PATH: &str = "sqlite://./db.sqlite?mode=rwc";
const SOCKET: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let db = Database::connect(DB_PATH).await.unwrap();
    assert!(db.ping().await.is_ok());

    let state = App::new(db).await;
    println!("Initial token: {}", state.add_registration_token().await);

    let app = Router::new()
        .nest("/api/v1", api::router().await)
        // TODO: properly implement route fallbacks
        .route("/login", get(format!("{}/index.html", FRONTEND)))
        .route("/register", get(format!("{}/index.html", FRONTEND)))
        .route(
            "/edit",
            get(Html(include_str!("../../frontend/dist/index.html"))),
        )
        .fallback_service(ServiceBuilder::new().service(ServeDir::new(FRONTEND)));
    let app = app.with_state(state).layer(CookieManagerLayer::new());

    let addr = SocketAddr::from_str(SOCKET).unwrap();
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

// Idea: use a CRDT for files. External services like DB and git sync act like any other client
//
// server manages:
// - users
// - prpjects
// - workers (The things on which the code compiles)
// -
