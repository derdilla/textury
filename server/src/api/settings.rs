use axum::{Json, extract::State};

use crate::{api, app::App};

pub(crate) async fn get_settings(State(app): State<App>) -> Json<api::types::ServerSettings> {
    let settings = app.settings();
    Json(api::types::ServerSettings {
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
