use crate::app::App;
use crate::model;
use api_types::*;
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
    Extension(user): Extension<model::user::Model>,
    Path(project_id): Path<String>,
    Json(metadata): Json<CreateProjectDetails>,
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

    // TODO: avoid leaking existing projects
    if app.project_exists(&project_id).await {
        return (StatusCode::CONFLICT, "Project already exists");
    }

    if app
        .insert_project(project_id, user.id, metadata.title)
        .await
    {
        return (StatusCode::OK, "Success");
    }

    (StatusCode::INTERNAL_SERVER_ERROR, "Project creation failed")
}

pub async fn delete_project(
    State(app): State<App>,
    Extension(user): Extension<model::user::Model>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    let success = app
        .delete_project(&project_id, &user)
        .await
        .unwrap_or(false);

    if !success {
        return (
            StatusCode::UNAUTHORIZED,
            "Unable to delete project, it either doesn't exist or you have insufficient permissions",
        );
    }

    (StatusCode::OK, "Ok")
}

pub async fn share_project(
    State(app): State<App>,
    Extension(user): Extension<model::user::Model>,
    Path(project_id): Path<String>,
) -> (StatusCode, String) {
    if !app.project_exists_for_user(&project_id, &user).await {
        return (StatusCode::NOT_FOUND, "Project not found".into());
    }

    let Some(token) = app.share(&project_id, &user).await else {
        return (StatusCode::NOT_FOUND, "Project not found".into());
    };

    (StatusCode::OK, token)
}

pub async fn accept_share(
    State(app): State<App>,
    Extension(user): Extension<model::user::Model>,
    Path(share_token): Path<String>,
) -> (StatusCode, &'static str) {
    match app.accept_share(&share_token, &user).await {
        true => (StatusCode::OK, "Project joined"),
        false => (StatusCode::NOT_FOUND, "Token not found"),
    }
}

pub(crate) async fn list_projects(
    State(app): State<App>,
    Extension(user): Extension<model::user::Model>,
) -> Response {
    let Some(projects) = app.list_projects(&user).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed listing projects").into_response();
    };

    let mut send_projects = Vec::new();
    for p in projects {
        let app = app.clone();
        send_projects.push(async move {
            let metadata = app.get_project_metadata_for_user(&p.id, user.id).await;
            let members = app
                .get_project_members(&p.id)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|u| User { id: u.id, name: u.name })
                .collect();
            ProjectSummary {
                id: p.id,
                title: p.title,
                owner: User {
                    id: p.owner_id,
                    name: app
                        .user_from_id(p.owner_id)
                        .await
                        .map(|user| user.name)
                        .unwrap_or("unknown".into()),
                },
                members,
                created_at: p.last_modified,
                updated_at: p.last_modified,
                edited_by: User {
                    id: p.owner_id,
                    name: app
                        .user_from_id(p.owner_id)
                        .await
                        .map(|user| user.name)
                        .unwrap_or("unknown".into()),
                },
                metadata: metadata,
            }
        });
    }

    (
        StatusCode::OK,
        Json(ProjectOverviewIndex {
            projects: future::join_all(send_projects).await,
        }),
    )
        .into_response()
}

pub async fn update_user_meta(
    State(app): State<App>,
    Extension(user): Extension<model::user::Model>,
    Path(project_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let Some(success) = app
        .set_project_metadata_for_user(&project_id, user.id, body)
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

pub async fn get_index(
    State(app): State<App>,
    Extension(user): Extension<model::user::Model>,
    Path(project_id): Path<String>,
) -> Response {
    let Some(project) = app.get_project_for_user(&project_id, &user).await else {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    };

    let members = app
        .get_project_members(&project.id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|u| User { id: u.id, name: u.name })
        .collect();

    let index = ProjectIndex {
        summary: ProjectSummary {
            id: project.id,
            title: project.title,
            owner: User {
                id: project.owner_id,
                name: app
                    .user_from_id(project.owner_id)
                    .await
                    .map(|user| user.name)
                    .unwrap_or("unknown".into()),
            },
            members,
            created_at: project.created_at,
            updated_at: project.last_modified,
            edited_by: User {
                id: project.owner_id,
                name: app
                    .user_from_id(project.owner_id)
                    .await
                    .map(|user| user.name)
                    .unwrap_or("unknown".into()),
            },
            metadata: app
                .get_project_metadata_for_user(&project_id, user.id)
                .await,
        },
    };

    Json(index).into_response()
}

pub async fn edit_ws(
    State(app): State<App>,
    Extension(user): Extension<model::user::Model>,
    Path(project_id): Path<String>,
    ws: WebSocketUpgrade,
) -> Response {
    if !app.project_exists_for_user(&project_id, &user).await {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    ws.on_upgrade(async move |socket| {
        let project = app.project(project_id).await;
        project.connect(socket).await;
    })
}

pub async fn last_compiled(
    State(app): State<App>,
    Extension(user): Extension<model::user::Model>,
    Path(project_id): Path<String>,
) -> Response<Body> {
    let Some(project) = app.get_project_for_user(&project_id, &user).await else {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    };

    let Some(latest_compilation) = project.latest_compilation else {
        return (StatusCode::NOT_FOUND, "No latest compilation").into_response();
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
    Extension(user): Extension<model::user::Model>,
    Path(project_id): Path<String>,
) -> Response {
    if !app.project_exists_for_user(&project_id, &user).await {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    let header = format!("attachment; filename=\"{}.zip\"", &project_id);
    let zip = ReaderStream::new(app.zip(project_id).await);
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

pub(crate) mod api_types {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ProjectOverviewIndex {
        pub projects: Vec<ProjectSummary>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ProjectSummary {
        pub id: String,
        pub title: String,
        pub owner: User,
        pub members: Vec<User>,
        pub created_at: time::PrimitiveDateTime,
        pub updated_at: time::PrimitiveDateTime,
        pub edited_by: User,
        /// User-specific metadata field.
        pub metadata: Option<serde_json::Value>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct User {
        pub id: i32,
        pub name: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CreateProjectDetails {
        pub title: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ProjectIndex {
        pub summary: ProjectSummary,
    }
}
