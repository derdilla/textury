use std::{collections::HashMap, str::FromStr, sync::Arc};

use axum::{
    Json, RequestPartsExt, Router,
    extract::{FromRequest, FromRequestParts, Path, Request},
    http::{self, StatusCode, Uri},
    middleware::Next,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use tower_sessions::Session;

const PROJECTS_KEY: &str = "projects";

pub fn router() -> Router {
    use axum::routing::{get, post};

    Router::new()
        .route("/{id}", /*get(get_project)*/ post(create_project))
        .route("/", get(list_projects))
}

struct ProjectManager {
    conn: Arc<rusqlite::Connection>,
}

impl ProjectManager {
    /// Creates a new project.
    ///
    /// - [id] must be a URI path segment
    fn create_project(&self, id: String, owner: User) -> Option<Project> {
        let existing = self.conn
            .execute("SELECT (id) FROM projects WHERE id = ?1", (&id,));
        self.conn.query_row(sql, params, f)

        None
    }
}

struct User {
    conn: Arc::new(rusqlite::Connection),
    id: String,
}

struct Project {
    conn: Arc::new(rusqlite::Connection),
    id: String,
}

/*
#[derive(Debug, Serialize, Deserialize, Validate)]
struct Project {
    #[validate(min_length = 3)]
    #[validate(max_length = 100)]
    #[validate(
        pattern = r"^[a-zA-Z0-9.-_~!$&'()*+,;=:@]*$",
        message = "Project ID must be URI path segment"
    )]
    pub id: String,
    pub owner_id: String,
    /// Fine grained access token that allows pushing and pulling the repository.
    pub git_token: String,
}

struct ProjectManager {
    /// List of projects the user is a member of.
    projects: Vec<Project>,
    session: Session,
}

impl<S> FromRequestParts<S> for ProjectManager
where
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(
        req: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(req, state).await?;
        let Ok(Some(projects)) = session.get::<Vec<Project>>(PROJECTS_KEY).await else {
            return Err((http::StatusCode::UNAUTHORIZED, "UNAUTHORIZED"));
        };

        Ok(ProjectManager { projects, session })
    }
}

/*
impl<S> FromRequestParts<S> for Project
where
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(
        req: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let projects = ProjectManager::from_request_parts(req, state).await?;

        let id = req
            .extract::<Path<HashMap<String, String>>>()
            .await
            .map(|Path(path_params)| path_params.get("id"))
            .ok()
            .flatten()
            .ok_or((
                StatusCode::BAD_REQUEST,
                "Missing parameter for project <id>",
            ))?;

        projects
            .into_list()
            .take(id)
            .ok_or((StatusCode::UNAUTHORIZED, "UNAUTHORIZED"))
    }
}*/

impl ProjectManager {
    pub fn into_list(self) -> Vec<Project> {
        self.projects
    }

    pub fn get(&self, id: &str) -> Option<&Project> {
        self.projects.iter().find(|e| &e.id == id)
    }

    pub async fn link(&mut self, project: Project) -> bool {
        self.projects.push(project);

        self.session
            .insert(PROJECTS_KEY, &self.projects)
            .await
            .is_ok()
    }
}

async fn create_project<'a>(
    mut projects: ProjectManager,
    Json(req): Json<CreateProjectRequest>,
) -> impl IntoResponse {
    // Check whether there are rw permissions to clone and commit to the repo. Some cross provider version of
    // https://docs.github.com/en/rest/collaborators/collaborators?apiVersion=2022-11-28#get-repository-permissions-for-a-user

    if projects.link(req).await {
        (StatusCode::OK, "Success")
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't create project")
    }
}

async fn list_projects(projects: ProjectManager) -> impl IntoResponse {
    Json(projects.into_list())
}

async fn open(_projects: ProjectManager) -> impl IntoResponse {
    "TODO: check perms and open stream"
}

#[derive(Debug, Deserialize)]
struct CreateProjectRequest<'a> {
    pub title: &'a str,
}
/*async fn get_project(
    projects: ProjectManager,
    Path(id): Path<&String>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let p = projects
        .get(id)
        .ok_or((StatusCode::NOT_FOUND, "No such project"))?;
    // TODO: very useless considering list exists
    Json(p.clone())
}*/
*/
