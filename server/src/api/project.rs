use crate::api::project::api::{ProjectIndex, ProjectSummary};
use crate::api::App;
use axum::extract::State;
use axum::response::Response;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json, Router};
use tower_cookies::Cookies;

pub async fn router(router: Router<App>) -> Router<App> {
    use axum::routing::{get, put};

    router
        .route("/project/{id}", /*get(get_project)*/ put(create_project))
        .route("/projects", get(list_projects))
}

async fn create_project(
    State(projects): State<App>,
    cookies: Cookies,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    let Some(user) = projects.user(cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in");
    };

    let re = regex::Regex::new(r"^[a-zA-Z0-9.-_~!$&'()*+,;=:@]*$").unwrap();
    if !re.is_match(&project_id) {
        return (StatusCode::BAD_REQUEST, "Project ID must be URI path segment");
    }

    // TODO: avoid leaking existing projects
    if projects.project_exists(&project_id).await {
        return (StatusCode::CONFLICT, "Project already exists");
    }

    if projects.insert_project(project_id, user.id).await {
        return (StatusCode::OK, "Success");
    }

    (StatusCode::INTERNAL_SERVER_ERROR, "Project creation failed")
}

async fn list_projects(
    State(app): State<App>,
    cookies: Cookies,
) -> Response {
    let Some(user) = app.user(cookies).await else {
        return (StatusCode::UNAUTHORIZED, "Not logged in").into_response();
    };

    let Some(projects) = app.list_projects(user).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed listing projects").into_response();
    };

    (StatusCode::OK, Json(ProjectIndex {
        projects: projects.iter()
            .map(|p| ProjectSummary { id : p.id.clone() })
            .collect(),
    })).into_response()
}

mod api {
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

/*

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
