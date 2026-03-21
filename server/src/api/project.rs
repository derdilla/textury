use crate::app::App;
use api_types::*;
use axum::body::Bytes;
use axum::extract::{State, WebSocketUpgrade};
use axum::response::Response;
use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use tower_cookies::Cookies;

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

pub async fn compile(
    State(app): State<App>,
    cookies: Cookies,
    Path(project_id): Path<String>,
) -> Response {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in").into_response();
    };

    if !app.project_exists_for_user(&project_id, &user).await {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    todo!() // Json<CompilationResult>
}

pub async fn get_index(
    State(app): State<App>,
    cookies: Cookies,
    Path(project_id): Path<String>,
) -> Response {
    let Some(user) = super::user(&app, cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in").into_response();
    };

    if !app.project_exists_for_user(&project_id, &user).await {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    let Some(files) = app.list_files(&project_id).await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Couldn't build file index",
        )
            .into_response();
    };

    Json(files).into_response()
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
    if app.add_or_update_file(project_id, file_path, body).await {
        (StatusCode::OK, "Success")
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't add file")
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

    if app.delete_file(project_id, file_path).await {
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

    match app.get_file(project_id, file_path).await {
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

    let document_id = format!("doc-{}-{}", project_id, file_path);
    ws.on_upgrade(async move |socket| app.open_document(socket, document_id).await)
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

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CompilationResult {
        pub log: String,
        pub document: Option<Vec<u8>>,
    }
}
