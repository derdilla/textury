use crate::app::App;
use api_types::*;
use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use tower_cookies::Cookies;
use tracing::{info, log};

pub(crate) async fn create_project(
    State(app): State<App>,
    cookies: Cookies,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in");
    };

    let re = regex::Regex::new(r"^[a-zA-Z0-9.-_~!$&'()*+,;=:@]*$").unwrap();
    if !re.is_match(&project_id) {
        return (
            StatusCode::BAD_REQUEST,
            "Project ID must be URI path segment",
        );
    }

    // TODO: avoid leaking existing projects
    if app.project_exists(&project_id).await {
        return (StatusCode::CONFLICT, "Project already exists");
    }

    if app.insert_project(project_id, user.id).await {
        return (StatusCode::OK, "Success");
    }

    (StatusCode::INTERNAL_SERVER_ERROR, "Project creation failed")
}

pub(crate) async fn list_projects(State(app): State<App>, cookies: Cookies) -> Response {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in").into_response();
    };

    let Some(projects) = app.list_projects(&user).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed listing projects").into_response();
    };

    (
        StatusCode::OK,
        Json(ProjectIndex {
            projects: projects
                .iter()
                .map(|p| ProjectSummary { id: p.id.clone() })
                .collect(),
        }),
    )
        .into_response()
}

#[axum::debug_handler]
pub(crate) async fn proxy_to_worker(
    State(app): State<App>,
    cookies: Cookies,
    Path((project_id, req_path)): Path<(String, String)>,
    req: axum::http::Request<Body>,
) -> Response<Body> {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in").into_response();
    };

    if !app.project_exists_for_user(&project_id, &user).await {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    let path = format!("http://worker.socket/{}", req_path);
    info!("proxying {path} to {project_id} worker");

    let Some(client) = app.get_worker(project_id).await else {
        log::error!("Failed to build socket client");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    // TODO: handle_ws_upgrade
    let Ok(response) = client
        .request(req.method().clone(), path)
        .body(reqwest::Body::wrap_stream(
            req.into_body().into_data_stream(),
        ))
        .send()
        .await
    else {
        log::error!("Failed to send proxied request");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    Response::builder()
        .status(response.status())
        .body(Body::from_stream(response.bytes_stream()))
        .unwrap()
}

pub(crate) mod api_types {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ProjectIndex {
        pub projects: Vec<ProjectSummary>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ProjectSummary {
        pub id: String,
    }
}
