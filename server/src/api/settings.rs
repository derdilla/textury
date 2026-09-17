use axum::{Json, extract::State};
use serde::Serialize;

use crate::app::App;

pub(crate) async fn get_settings(State(app): State<App>) -> Json<ApiServerSettings> {
    let settings = app.settings();
    Json(ApiServerSettings {
        open_sign_ups: settings.open_sign_ups(),
        user_projects: settings.user_projects(),
        open_sharing: settings.open_sharing(),
    })
}

pub(crate) async fn set_open_sign_ups(
    State(app): State<App>,
    Json(value): Json<bool>,
) -> Result<(), String> {
    app.settings().set_open_sign_ups(value).await
}
pub(crate) async fn set_user_projects(
    State(app): State<App>,
    Json(value): Json<bool>,
) -> Result<(), String> {
    app.settings().set_user_projects(value).await
}
pub(crate) async fn set_open_sharing(
    State(app): State<App>,
    Json(value): Json<bool>,
) -> Result<(), String> {
    app.settings().set_open_sharing(value).await
}

#[derive(Debug, Serialize)]
pub(crate) struct ApiServerSettings {
    pub open_sign_ups: bool,
    pub user_projects: bool,
    pub open_sharing: bool,
}
