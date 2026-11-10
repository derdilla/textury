
use crate::api::types::{self, CreatePublicShareRequest, ProjectId, UserId};
use crate::app::{self, App};
use crate::token_manager;
use axum::Extension;
use axum::body::{Body, Bytes};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::{HeaderValue, header};
use axum::response::Response;
use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use futures::future;
use tokio_util::io::ReaderStream;
use tracing::warn;

pub(crate) async fn create_project(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Json(metadata): Json<types::CreateProjectDetails>,
) -> Response {
    if let Some(pid) = app.insert_project(user.id, metadata.title).await {
        return (StatusCode::OK, Json(pid)).into_response();
    }

    StatusCode::UNAUTHORIZED.into_response()
}

pub async fn delete_project(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<ProjectId>,
) -> impl IntoResponse {
    let result = app
        .delete_project(project_id, user.id)
        .await;

    match result {
        app::DeleteProjectResult::NotFound
        | app::DeleteProjectResult::NotAlllowed => (
            StatusCode::UNAUTHORIZED,
            "Unable to delete project, it either doesn't exist or you have insufficient permissions",
        ),
        app::DeleteProjectResult::SuccessfullyDeleted => (StatusCode::OK, "Project deleted"),
    }
}

pub async fn share_project(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<ProjectId>,
    Json(initial_permissions): Json<types::ProjectPermissionLevel>,
) -> Response {
    if !app.project_exists_for_user(project_id, user.id).await {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    let Some(token) = app.share(project_id, user.id, initial_permissions).await else {
        return (StatusCode::UNAUTHORIZED, "Insufficient permissions").into_response();
    };

    Json(types::ProjectInvite {
        token,
        expires: token_manager::DEFAULT_PROJECT_JOIN_EXPIRERY.whole_minutes(),
    }).into_response()
}

pub async fn share_project_publicly(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<ProjectId>,
    Json(request): Json<CreatePublicShareRequest>,
) -> Response {
    if !app.project_exists_for_user(project_id, user.id).await {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    Json(app.create_public_share(
        project_id,
        user.id,
        request.share_id,
        request.share_password,
        request.initial_permissions,
    ).await).into_response()
}

pub async fn get_public_share_info(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<ProjectId>,
) -> Response {
    let Some(share) = app.get_public_share(project_id, user.id).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    Json(share).into_response()
}

pub async fn unshare_publicly(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<ProjectId>,
) -> StatusCode {
    match app.unshare_publicly(project_id, user.id).await {
        true => StatusCode::OK,
        false => StatusCode::NOT_FOUND,
    }
}

pub async fn set_title(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<ProjectId>,
    Json(new_title): Json<String>,
) -> (StatusCode, &'static str) {
    match app.change_title(project_id, user.id, new_title).await {
        true => (StatusCode::OK, "Title changed successfully"),
        false => (StatusCode::UNAUTHORIZED, "Project does not exist or user isn't authorized"),
    }
}

pub async fn preview_share(
    State(app): State<App>,
    Json(share_token): Json<types::JoinProjectRequest>,
) -> Response {
    match app.preview_share(&share_token.token).await {
        Some(value) => (StatusCode::OK, Json(value)).into_response(),
        None => (StatusCode::UNAUTHORIZED, "Invalid or expired token").into_response(),
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

pub async fn leave(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<ProjectId>,
) -> StatusCode {
    let Some(project) = app.get_project_for_user(project_id, user.id).await else {
        return StatusCode::NOT_FOUND;
    };
    if project.owner_id == user.id {
        return StatusCode::CONFLICT;
    }

    app.remove_user_from_project(project_id, user.id).await;
    StatusCode::OK
}

pub async fn members(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<ProjectId>,
) -> Response {
    if app.get_project_for_user(project_id, user.id).await.is_none() {
        return StatusCode::NOT_FOUND.into_response();
    };

    let Some(members) = app.get_project_member_permissions(project_id).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let members = members.map(|m| types::ProjectMemberInfo {
        id: m.0,
        permissions: m.1,
    }).collect::<Vec<_>>();
    Json(types::MemberPermissions(members)).into_response()
}

pub async fn transfer_ownership(
    State(app): State<App>,
    Extension(requester): Extension<app::UserPrivateSummary>,
    Path((project_id, user_id)): Path<(ProjectId, UserId)>,
) -> StatusCode {
    let Some(project) = app.get_project_for_user(project_id, requester.id).await else {
        return StatusCode::NOT_FOUND;
    };

    if project.owner_id != requester.id {
        return StatusCode::UNAUTHORIZED;
    }

    match app.transfer_ownership(project_id, user_id).await {
        app::TransferOwnershipResult::Success => StatusCode::OK,
        app::TransferOwnershipResult::NewOwnerNotMember => StatusCode::NOT_FOUND,
        app::TransferOwnershipResult::NewOwnerIsTemporary => StatusCode::CONFLICT,
    }
}

pub async fn kick(
    State(app): State<App>,
    Extension(requester): Extension<app::UserPrivateSummary>,
    Path((project_id, user_id)): Path<(ProjectId, UserId)>,
) -> StatusCode {
    let Some(project) = app.get_project_for_user(project_id, requester.id).await else {
        return StatusCode::NOT_FOUND;
    };

    if project.owner_id != requester.id {
        return StatusCode::UNAUTHORIZED;
    }

    if project.owner_id == user_id {
        return StatusCode::CONFLICT;
    }

    app.kick(project_id, user_id).await;
    StatusCode::OK
}

pub async fn change_permissions(
    State(app): State<App>,
    Extension(requester): Extension<app::UserPrivateSummary>,
    Path((project_id, user_id)): Path<(ProjectId, UserId)>,
    Json(permissions): Json<types::ProjectPermissionLevel>
) -> StatusCode {
    let Some(project) = app.get_project_for_user(project_id, requester.id).await else {
        return StatusCode::NOT_FOUND;
    };

    if project.owner_id != requester.id {
        return StatusCode::UNAUTHORIZED;
    }

    app.change_permissions(project_id, user_id, permissions).await;
    StatusCode::OK
}

pub(crate) async fn list_projects(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
) -> Response {
    let Some(projects) = app.list_projects(user.id).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed listing projects").into_response();
    };
    (
        StatusCode::OK,
        Json(compile_projects_response(app, user.id, projects).await),
    )
        .into_response()
}

pub async fn update_user_meta(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<ProjectId>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let Some(success) = app
        .set_project_metadata_for_user(project_id, user.id, serde_json::to_string(&body).expect("Value just decoded"))
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
    Path(project_id): Path<ProjectId>,
    ws: WebSocketUpgrade,
) -> Response {
    if !app.project_exists_for_user(project_id, user.id).await {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    ws.on_upgrade(async move |socket| {
        let project = app.project(project_id).await;
        project.connect(socket, user.id).await;
    })
}

pub async fn last_compiled(
    State(app): State<App>,
    Extension(user): Extension<app::UserPrivateSummary>,
    Path(project_id): Path<ProjectId>,
) -> Response<Body> {
    let Some(latest_compilation) = app.get_latest_compilation_for_user(project_id, user.id).await else {
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
    Path(project_id): Path<ProjectId>,
) -> Response {
    if !app.project_exists_for_user(project_id, user.id).await {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    let header = format!("attachment; filename=\"{}.zip\"", project_id.to_string());
    let (reader, writer) = tokio::io::duplex(1024 * 8);
    tokio::spawn(async move {
        app.zip(project_id, writer).await
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

pub(crate) async fn compile_projects_response(
    app: App,
    user_id: UserId,
    projects: Vec<app::ProjectModel>,
) -> Vec<types::ProjectSummary> {
    let mut send_projects = Vec::new();
    for p in projects {
        let app = app.clone();
        send_projects.push(async move {
            let user_specific = app.get_project_metadata_for_user(p.id, user_id).await;
            if user_specific.is_none() {
                warn!("user_specific is None for {}, {}", user_id, p.id);
            }
            let metadata = user_specific
                .as_ref()
                .and_then(|u| u.1.as_deref())
                .map(|s| serde_json::from_str(s).expect("DB corrupt"))
                .unwrap_or(serde_json::Map::new());
            let members = app.get_project_member_ids(p.id).await
                .map(|list| list.collect())
                .unwrap_or_default();
            types::ProjectSummary {
                id: p.id,
                title: p.title,
                owner: p.owner_id.into(),
                members,
                created_at: p.created_at.0,
                updated_at: p.last_modified.0,
                updated_by: p.last_modified_by.into(),
                metadata: types::UserMeta(metadata),
                permissions: user_specific.map(|v| v.0)
                    .unwrap_or(types::ProjectPermissionLevel::View),
            }
        });
    }
    future::join_all(send_projects).await
}
