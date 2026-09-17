
use crate::api::types;
use crate::app::{self, App, ProjectID};
use crate::token_manager;
use axum::Extension;
use axum::body::{Body, Bytes};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::{HeaderValue, header};
use axum::response::Response;
use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use futures::future;
use tokio_util::io::ReaderStream;

pub(crate) async fn create_project(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<String>,
    Json(metadata): Json<types::CreateProjectDetails>,
) -> impl IntoResponse {
    // Prohibit stuff that's bad for URLs
    let re = regex::Regex::new(r"^[a-zA-Z0-9.-_~!$&'()*+,;=:@]*$").unwrap();
    if !re.is_match(&project_id) {
        return (
            StatusCode::BAD_REQUEST,
            "Project ID must be URI path segment",
        );
    }

    // Prohibit stuff that's bad for saving to disk
    if project_id.contains(".") || project_id.contains("/") {
        return (
            StatusCode::BAD_REQUEST,
            "Project ID may not contain dots and /",
        );
    }
    let project = app::ProjectID(project_id);

    if app.project_exists(&project).await {
        return (StatusCode::CONFLICT, "Project already exists");
    }

    if app.insert_project(&project, user.id, metadata.title).await {
        return (StatusCode::OK, "Success");
    }

    (StatusCode::INTERNAL_SERVER_ERROR, "Project creation failed")
}

pub async fn delete_project(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    let project_id = ProjectID(project_id);
    let success = app
        .delete_project(&project_id, user.id)
        .await;

    if !success {
        return (
            StatusCode::UNAUTHORIZED,
            "Unable to delete project, it either doesn't exist or you have insufficient permissions",
        );
    }

    (StatusCode::OK, "Project deleted")
}

pub async fn share_project(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<String>,
) -> Response {
    let project_id = ProjectID(project_id);
    if !app.project_exists_for_user(&project_id, user.id).await {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    let Some(token) = app.share(&project_id, user.id).await else {
        return (StatusCode::UNAUTHORIZED, "Insufficient permissions").into_response();
    };

    Json(types::ProjectInvite {
        token,
        expires: token_manager::DEFAULT_PROJECT_JOIN_EXPIRERY.whole_minutes(),
    }).into_response()
}

pub async fn set_title(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<String>,
    Json(new_title): Json<String>,
) -> (StatusCode, &'static str) {
    let project_id = ProjectID(project_id);

    match app.change_title(&project_id, user.id, new_title).await {
        true => (StatusCode::OK, "Title changed successfully"),
        false => (StatusCode::UNAUTHORIZED, "Project does not exist or user isn't authorized"),
    }
}

pub async fn accept_share(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Json(share_token): Json<types::JoinProjectRequest>,
) -> (StatusCode, &'static str) {
    match app.accept_share(&share_token.token, user.id).await {
        true => (StatusCode::OK, "Project joined"),
        false => (StatusCode::UNAUTHORIZED, "Invalid or expired token"),
    }
}

pub(crate) async fn list_projects(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
) -> Response {
    let Some(projects) = app.list_projects(user.id).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed listing projects").into_response();
    };

    let mut send_projects = Vec::new();
    for p in projects {
        let app = app.clone();
        send_projects.push(async move {
            let metadata = app.get_project_metadata_for_user(&p.id, user.id).await;
            let metadata = metadata.map(|v| serde_json::from_str(&v).expect("DB corrupt"))
                .unwrap_or(serde_json::Map::new());
            let members = app
                .get_project_members(&p.id)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|u| u.id.into())
                .collect();
            types::ProjectSummary {
                id: p.id.into(),
                title: p.title,
                owner: p.owner_id.into(),
                members,
                created_at: p.created_at.0,
                updated_at: p.last_modified.0,
                updated_by: p.last_modified_by.into(),
                metadata: types::UserMeta(metadata),
            }
        });
    }

    (
        StatusCode::OK,
        Json(future::join_all(send_projects).await),
    )
        .into_response()
}

pub async fn update_user_meta(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let project_id = ProjectID(project_id);
    let Some(success) = app
        .set_project_metadata_for_user(&project_id, user.id, serde_json::to_string(&body).expect("Value just decoded"))
        .await
    else {
        return (StatusCode::NOT_FOUND, "Project not found");
    };

    if success {
        return (StatusCode::OK, "Metadata updated");
    }
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Unable to update metadata",
    )
}

pub async fn edit_ws(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<String>,
    ws: WebSocketUpgrade,
) -> Response {
    let project_id = ProjectID(project_id);
    if !app.project_exists_for_user(&project_id, user.id).await {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    ws.on_upgrade(async move |socket| {
        let project = app.project(&project_id).await;
        project.connect(socket, user.id).await;
    })
}

pub async fn last_compiled(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<String>,
) -> Response<Body> {
    let project_id = ProjectID(project_id);
    let Some(latest_compilation) = app.get_latest_compilation_for_user(&project_id, user.id).await else {
        return (StatusCode::NOT_FOUND, "No project or latest compilation found").into_response();
    };

    let mut response = Bytes::from_owner(latest_compilation).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    response
}

pub async fn zip(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<String>,
) -> Response {
    let project_id = ProjectID(project_id);
    if !app.project_exists_for_user(&project_id, user.id).await {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    let header = format!("attachment; filename=\"{}.zip\"", &project_id);
    let (reader, writer) = tokio::io::duplex(1024 * 8);
    tokio::spawn(async move {
        app.zip(&project_id, writer).await
    });
    let zip = ReaderStream::new(reader);
    let body = Body::from_stream(zip);
    let mut response = body.into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/zip"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&header).expect("Project id not correctly validated"),
    );
    response
}
