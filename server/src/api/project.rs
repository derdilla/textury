use crate::app::App;
use api_types::*;
use axum::body::{Body, Bytes};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::{HeaderValue, header};
use axum::response::Response;
use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use futures::future;
use tower_cookies::Cookies;

pub(crate) async fn create_project(
    State(app): State<App>,
    cookies: Cookies,
    Path(project_id): Path<String>,
    Json(metadata): Json<CreateProjectDetails>,
) -> impl IntoResponse {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in");
    };

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
    cookies: Cookies,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in");
    };

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
    cookies: Cookies,
    Path(project_id): Path<String>,
) -> (StatusCode, String) {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in".into());
    };

    if !app.project_exists_for_user(&project_id, &user).await {
        return (StatusCode::NOT_FOUND, "Project not found".into());
    }

    let Some(token) = app.share(&project_id, &user).await else {
        return (StatusCode::NOT_FOUND, "Project not found".into());
    };

    return (StatusCode::OK, token);
}

pub async fn accept_share(
    State(app): State<App>,
    cookies: Cookies,
    Path(share_token): Path<String>,
) -> (StatusCode, &'static str) {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in");
    };

    match app.accept_share(&share_token, &user).await {
        true => (StatusCode::OK, "Project joined"),
        false => (StatusCode::NOT_FOUND, "Token not found"),
    }
}

pub(crate) async fn list_projects(State(app): State<App>, cookies: Cookies) -> Response {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in").into_response();
    };

    let Some(projects) = app.list_projects(&user).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed listing projects").into_response();
    };

    let mut send_projects = Vec::new();
    for p in projects {
        let app = app.clone();
        send_projects.push(async move {
            let metadata = app.get_project_metadata_for_user(&p.id, user.id).await;
            ProjectSummary {
                id: p.id,
                title: p.title,
                last_modified: p.last_modified,
                owner: User {
                    id: p.owner_id,
                    name: app
                        .user_from_id(p.owner_id)
                        .await
                        .map(|user| user.name)
                        .unwrap_or("unknown".into()),
                },
                metadata,
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
#[axum::debug_handler]
pub async fn update_user_meta(
    State(app): State<App>,
    cookies: Cookies,
    Path(project_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in");
    };

    let Some(success) = app
        .set_project_metadata_for_user(&project_id, user.id, body)
        .await
    else {
        return (StatusCode::NOT_FOUND, "Project not found");
    };

    if success {
        return (StatusCode::OK, "Metadata updated");
    }
    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Unable to update metadata",
    );
}

pub async fn get_index(
    State(app): State<App>,
    cookies: Cookies,
    Path(project_id): Path<String>,
) -> Response {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in").into_response();
    };

    let Some(project) = app.get_project_for_user(&project_id, &user).await else {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    };

    let Some(files) = app.list_files(&project_id).await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Couldn't build file index",
        )
            .into_response();
    };

    let index = ProjectIndex {
        summary: ProjectSummary {
            id: project.id,
            title: project.title,
            last_modified: project.last_modified,
            owner: User {
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
        root: files,
    };

    Json(index).into_response()
}

pub async fn add_or_update_file(
    State(app): State<App>,
    cookies: Cookies,
    Path((project_id, file_path)): Path<(String, String)>,
    body: Bytes,
) -> (StatusCode, &'static str) {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in");
    };

    if !app.project_exists_for_user(&project_id, &user).await {
        return (StatusCode::NOT_FOUND, "Project not found");
    }

    let body = body.to_vec();
    if app.add_or_update_file(&project_id, &file_path, body).await {
        (StatusCode::OK, "Success")
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't add file")
    }
}

pub async fn move_file(
    State(app): State<App>,
    cookies: Cookies,
    Path((project_id, file_path)): Path<(String, String)>,
    destination: String,
) -> (StatusCode, &'static str) {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in");
    };

    if !app.project_exists_for_user(&project_id, &user).await {
        return (StatusCode::NOT_FOUND, "Project not found");
    }

    if app.move_file(&project_id, &file_path, &destination).await {
        (StatusCode::OK, "Success")
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't move file")
    }
}

pub async fn delete_file(
    State(app): State<App>,
    cookies: Cookies,
    Path((project_id, file_path)): Path<(String, String)>,
) -> (StatusCode, &'static str) {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in");
    };

    if !app.project_exists_for_user(&project_id, &user).await {
        return (StatusCode::NOT_FOUND, "Project not found");
    }

    if app.delete_entry(&project_id, &file_path).await {
        (StatusCode::OK, "Success")
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't delete file")
    }
}

pub async fn get_file(
    State(app): State<App>,
    cookies: Cookies,
    Path((project_id, file_path)): Path<(String, String)>,
) -> Response {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in").into_response();
    };

    if !app.project_exists_for_user(&project_id, &user).await {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    match app.get_file(&project_id, &file_path).await {
        Some(data) => data.into_response(),
        None => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

pub async fn yrs(
    State(app): State<App>,
    cookies: Cookies,
    Path((project_id, file_path)): Path<(String, String)>,
    ws: WebSocketUpgrade,
) -> Response {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in").into_response();
    };

    if !app.project_exists_for_user(&project_id, &user).await {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    let document_id = (project_id, file_path);
    ws.on_upgrade(async move |socket| app.open_document(socket, document_id).await)
}

pub async fn compile(
    State(app): State<App>,
    cookies: Cookies,
    Path((project_id, file_path)): Path<(String, String)>,
    Json(options): Json<CompileOptions>,
) -> Response {
    use base64::prelude::*;

    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in").into_response();
    };

    if !app.project_exists_for_user(&project_id, &user).await {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    let output = app.compile(project_id, file_path, options.compiler).await;
    let encoded = output.document.map(|data| {
        format!(
            "data:application/pdf;base64,{}",
            BASE64_STANDARD.encode(data)
        )
    });

    Json(api_types::CompilationResult {
        log: output.log,
        document: encoded,
    })
    .into_response()
}

pub async fn last_compiled(
    State(app): State<App>,
    cookies: Cookies,
    Path(project_id): Path<String>,
) -> Response<Body> {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in").into_response();
    };

    let Some(project) = app.get_project_for_user(&project_id, &user).await else {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    };

    let Some(latest_compilation) = project.latest_compilation else {
        return (StatusCode::NOT_FOUND, "No latest compilation").into_response();
    };

    let mut response = Bytes::from_owner(latest_compilation).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str("application/pdf").unwrap(),
    );
    response
}

pub(crate) mod api_types {
    use serde::{Deserialize, Serialize};

    use crate::{app::FileMetadata, compilation::Compiler};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ProjectOverviewIndex {
        pub projects: Vec<ProjectSummary>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ProjectSummary {
        pub id: String,
        pub title: String,
        pub owner: User,
        pub last_modified: time::PrimitiveDateTime,
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
    pub struct CompileOptions {
        pub compiler: Option<Compiler>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ProjectIndex {
        pub summary: ProjectSummary,
        pub root: FileMetadata,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CompilationResult {
        pub log: String,
        /// data URL of the document.
        pub document: Option<String>,
    }
}
