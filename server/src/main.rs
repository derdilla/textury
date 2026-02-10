mod api;
mod model;
mod users;
mod workers;

use std::net::SocketAddr;

use crate::workers::WorkerManager;
use axum::{http::StatusCode, response::IntoResponse, routing::get};
use time::Duration;
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};

#[tokio::main]
async fn main() {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::hours(10)));

    let app = api::router() // FIXME: implement
        .layer(session_layer);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    let mut worker_manager = WorkerManager::default();
    let w = worker_manager
        .get_worker("https://codeberg.org/derdilla/AOC25.git".into())
        .await;
}

// todo persistence

// Idea: use a CRDT for files. External services like DB and git sync act like any other client
//
// server manages:
// - users
// - prpjects
// - workers (The things on which the code compiles)
// -
