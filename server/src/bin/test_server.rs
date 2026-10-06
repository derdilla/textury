use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use server::{api, app::App, config};
use tower_cookies::CookieManagerLayer;
use tracing::warn;

use core::net::SocketAddr;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let config = config::Config {
        socket: "127.0.0.1:3000".into(),
        db_path: "sqlite::memory:".into(),
        fireframe_socket: None,
        idm_provides: Vec::new() };
    let state = App::new(
        &config.db_path,
        #[cfg(feature = "fireframe")]
        Arc::new(FireframeManager::new(config.fireframe_socket.unwrap()).await
            .expect("server compiled to sandbox, but connection to fireframe failed")),
    ).await.expect("App start failed");

    let app = Router::new()
        .route("/testGetInitialRegistrationToken", get(async |State(app): State<App>| {
            warn!("testGetInitialRegistrationToken issues registration token");
            return Json(app.add_registration_token(None).await);
        }))
        .route("/health", get("running"))
        .nest("/api/v1", api::router(state.clone()).await);
    let app = app.with_state(state).layer(CookieManagerLayer::new());

    let addr = SocketAddr::from_str(&config.socket).unwrap();
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();


}
