use crate::app::App;
use api_types::*;
use axum::extract::State;
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
