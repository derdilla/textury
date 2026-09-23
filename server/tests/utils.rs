use std::{net, str::FromStr};

use tower_cookies::CookieManagerLayer;

/// Start a test server returning API-url an registration token.
pub async fn test_server() -> TestServer {
    let app = server::app::App::new("sqlite::memory:")
        .await.unwrap();
    let token = app.add_registration_token(None).await;
    let state = app.clone();
    let app = axum::Router::new()
        .nest("/api/v1", server::api::router(app).await)
        .with_state(state)
        .layer(CookieManagerLayer::new());

    let addr = net::SocketAddr::from_str("127.0.0.1:0").unwrap();
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let url = format!("http://127.0.0.1:{}/api/v1", port);

    let handle = tokio::spawn(async move {
        axum::serve(listener, app.into_make_service()).await.unwrap();
    });

    TestServer {
        url,
        token,
        server: handle.abort_handle(),
    }
}

pub struct TestServer {
    pub url: String,
    pub token: String,
    server: tokio::task::AbortHandle,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.server.abort();
    }
}

pub mod client {
    include!(concat!(env!("OUT_DIR"), "/api_client.rs"));
}
