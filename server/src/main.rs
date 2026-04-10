mod api;
mod app;
mod compilation;
mod fs;
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

    let mut opt = ConnectOptions::new(DB_PATH);
    (&mut opt).sqlx_logging_level(log::LevelFilter::Trace);
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
}
