use crate::app::App;
use api_types::*;
use axum::extract::ws::WebSocket;
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

// TODO: testing function; remove
pub(crate) async fn edit_readme(
    ws: WebSocketUpgrade,
    State(app): State<App>,
    Path(project_id): Path<String>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, project_id, app))
}

async fn handle_socket(socket: WebSocket, project_id: String, app: App) {
    app.open_document(socket, project_id).await;
    /*
    // The client needs to send its current state to receive an update
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(client_state) => {
                let update = app.doc_state_as_update(&project_id, &client_state.into_data());
                if let Some(update) = update {
                    if socket.send(Message::Binary(update.into())).await.is_err() {
                        return;
                    }
                }
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    } */
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
