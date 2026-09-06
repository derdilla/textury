mod api;
mod app;
mod compilation;
mod config;
mod model;
mod websocket;
mod project;

use crate::app::App;
#[cfg(feature="fireframe")]
use crate::project::sandbox_manger::SandboxManager;
use axum::Router;
use sea_orm::{ConnectOptions, Database};
#[cfg(feature="fireframe")]
use std::sync::Arc;
use std::time::Duration;
use std::{net::SocketAddr, str::FromStr};
use tower_cookies::CookieManagerLayer;
use tracing::{info, log};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = config::Config::get();
    info!("Starting with configuration: {:#?}", &config);

    let mut opt = ConnectOptions::new(&config.db_path);
    opt.sqlx_logging_level(log::LevelFilter::Trace);
    let db = Database::connect(opt).await.unwrap();
    assert!(db.ping().await.is_ok());

    let state = App::new(
        db,
        #[cfg(feature = "fireframe")]
        Arc::new(SandboxManager::new(config.fireframe_socket.into()).await
            .expect("server compiled to sandbox, but connection to fireframe failed")),
    ).await;
    if state.user_count().await == 0 {
        println!(
            "{}\nInitial token: {}\n{}",
            "=".repeat(40),
            state.add_registration_token(None).await,
            "=".repeat(40)
        );
    }
    let cleanup_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_hours(6));
        loop {
            interval.tick().await;
            cleanup_state.gc().await;
        }
    });

    let app = Router::new().nest("/api/v1", api::router(state.clone()).await);
    let app = app.with_state(state).layer(CookieManagerLayer::new());

    let addr = SocketAddr::from_str(&config.socket).unwrap();
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
