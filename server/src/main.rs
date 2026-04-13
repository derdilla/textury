mod api;
mod app;
mod compilation;
mod fs;
mod model;
mod config;

use crate::app::App;
use axum::Router;
use sea_orm::{ConnectOptions, Database};
use std::{net::SocketAddr, str::FromStr};
use std::path::PathBuf;
use log::info;
use tower_cookies::CookieManagerLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = config::Config::get();
    info!("Starting with configuration: {:#?}", &config);

    let mut opt = ConnectOptions::new(&config.db_path);
    (&mut opt).sqlx_logging_level(log::LevelFilter::Trace);
    let db = Database::connect(opt).await.unwrap();
    assert!(db.ping().await.is_ok());

    let projects_root = PathBuf::from(config.projects_root).canonicalize().unwrap();

    let state = App::new(db, projects_root).await;
    if state.user_count().await == 0 {
        println!("{}\nInitial token: {}\n{}", "=".repeat(40), state.add_registration_token(None).await, "=".repeat(40));
    }

    let app = Router::new().nest("/api/v1", api::router().await);
    let app = app.with_state(state).layer(CookieManagerLayer::new());

    let addr = SocketAddr::from_str(&config.socket).unwrap();
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
