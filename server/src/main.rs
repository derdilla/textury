mod api;
mod app;
mod compilation;
mod model;

use crate::app::App;
use axum::Router;
use sea_orm::{ConnectOptions, Database};
use std::{net::SocketAddr, str::FromStr};
use tower_cookies::CookieManagerLayer;

const DB_PATH: &str = "sqlite://./db.sqlite?mode=rwc";
const SOCKET: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let opt = ConnectOptions::new(DB_PATH);
    let db = Database::connect(opt).await.unwrap();
    assert!(db.ping().await.is_ok());

    let state = App::new(db).await;
    println!("Initial token: {}", state.add_registration_token().await);

    let app = Router::new().nest("/api/v1", api::router().await);
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
