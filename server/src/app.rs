use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::api::types::{ProjectId, UserId};
#[cfg(feature = "fireframe")]
use crate::project::fireframe_manager;
#[cfg(not(feature = "fireframe"))]
use crate::project::{local_sandbox_manager};
use crate::settings::ServerSettings;
use crate::token_manager::TokenManager;
use crate::{api, websocket};
use sqlx::{Sqlite, SqlitePool};
use time::OffsetDateTime;
use tokio::io::DuplexStream;
use tokio::sync::{RwLock};
use tracing::{error, info, warn};

#[cfg(feature = "fireframe")]
type Manager = fireframe_manager::FireframeManager;
#[cfg(not(feature = "fireframe"))]
type Manager = local_sandbox_manager::LocalSandboxManager;

#[derive(Clone)]
pub struct App {
    conn: SqlitePool,
    token_manager: Arc<TokenManager>,
    projects: Arc<RwLock<HashMap<ProjectId, Arc<websocket::WebsocketProject<Manager>>>>>,
    settings: Arc<ServerSettings>,
    sandbox_manager: Arc<Manager>,
}

impl App {
    pub async fn new(
        db_path: &str,
        #[cfg(feature = "fireframe")]
        sandbox_manager: Arc<Manager>
    ) -> Result<Self, String>
    {
        let conn = Self::initialize_db(db_path).await?;
        #[cfg(not(feature = "fireframe"))]
        let sandbox_manager = Arc::new(local_sandbox_manager::LocalSandboxManager::new().await);
        Ok(Self {
            token_manager: Arc::new(TokenManager::new(conn.clone())),
            settings: Arc::new(ServerSettings::new(conn.clone()).await),
            sandbox_manager,
            projects: Default::default(),
            conn,
        })
    }

    #[cfg(test)]
    pub async fn test_db() -> Result<SqlitePool, String> {
        Self::initialize_db("sqlite::memory:").await
    }

    async fn initialize_db(db_path: &str) -> Result<SqlitePool, String> {
        let conn = SqlitePool::connect(db_path).await.unwrap();
        //assert!(conn.ping().await.is_ok());

        // Good for performance: https://sqlite.org/lang_analyze.html
        sqlx::query("PRAGMA optimize=0x10002;").execute(&conn).await.unwrap();
        {
            let conn = conn.clone();
            tokio::task::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_hours(20));

                loop {
                    interval.tick().await;
                    if let Err(e) = sqlx::query("PRAGMA optimize;").execute(&conn).await {
                        error!("PRAGMA optimize failed: {e}");
                    }
                }
            });
        }

        sqlx::migrate!("./migrations").run(&conn).await.unwrap();
        sqlx::query("PRAGMA optimize;").execute(&conn).await.unwrap();

        Ok(conn)
    }

    pub fn settings(&self) -> &ServerSettings {
        &self.settings
    }

    pub async fn all_users(&self) -> Vec<UserPublicSummary> {
        sqlx::query_as!(UserPublicSummary, r#"SELECT id, name FROM users"#)
            .fetch_all(&self.conn).await.unwrap()
    }

    pub async fn user_count(&self) -> u64 {
        match sqlx::query!("SELECT COUNT(id) as count from users").fetch_one(&self.conn).await {
            Ok(val) => val.count.try_into().expect("Negative COUNT"),
            Err(err) => {
                error!("Failed to query user count: {err}");
                0
            },
        }
    }

    #[cfg(all(test, not(feature = "fireframe")))]
    pub async fn project_count(&self) -> u64 {
        sqlx::query!("SELECT COUNT(id) as count from projects")
            .fetch_one(&self.conn)
            .await.unwrap()
            .count.try_into().expect("COUNT is negative")
    }

    /// Create a registration token, if [inviter] is administrator.
    pub async fn create_invite(&self, inviter: UserPrivateSummary) -> Option<String> {
        if !inviter.is_admin {
            return None;
        }

        Some(self.add_registration_token(Some(inviter.id)).await)
    }

    pub fn add_registration_token(&self, issuer_id: Option<UserId>) -> impl Future<Output = String> {
        self.token_manager.add_registration_token(issuer_id)
    }

    pub async fn register(
        &self,
        registration_token: String,
        name: String,
        email: String,
        password: String,
    ) -> CreateAccountResult {
        use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
        use crate::token_manager::TokenUseResult;

        if name.len() < 4 || password.len() < 8 {
            return CreateAccountResult::BadRequest;
        }

        if sqlx::query!("SELECT id FROM users WHERE email = ?", email)
            .fetch_optional(&self.conn)
            .await
            .is_ok_and(|res| res.is_some()) {
            return CreateAccountResult::Exists;
        }

        let issuer = {
            if self.settings.open_sign_ups() {
                None
            } else {
                match self.token_manager.try_use_registration_token(&registration_token).await {
                    TokenUseResult::NotFound => return CreateAccountResult::InvalidToken,
                    TokenUseResult::Expired => return CreateAccountResult::ExpiredToken,
                    TokenUseResult::Valid(issuer) => issuer,
                }
            }
        };

        // Save account
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        let argon2 = Argon2::default();
        let Ok(password_hash) = argon2.hash_password(password.as_bytes(), &salt) else {
            return CreateAccountResult::BadRequest;
        };
        let password_hash = password_hash.serialize().to_string();
        let is_admin = issuer.is_none();
        let uid = UserId::random();
        sqlx::query!(r#"INSERT INTO users (id, name, email, password_hash, is_admin, invited_by)
            VALUES (?, ?, ?, ?, ?, ?)"#,
            uid,
            name,
            email,
            password_hash,
            is_admin,
            issuer,
        ).execute(&self.conn).await.unwrap();

        CreateAccountResult::Created { id: uid }
    }

    pub async fn delete_user(&self, user_id: UserId) {
        let mut transaction = self.conn.begin().await.unwrap();

        // Leave all projects
        let projects = sqlx::query!("SELECT project_id
            FROM project_memberships WHERE user_id = ?",
            user_id)
            .fetch_all(&mut *transaction)
            .await.unwrap();
        for p in projects {
            match remove_user_from_project(&mut transaction, p.project_id, user_id).await {
                Ok(_) => {},
                Err(e) => {
                    error!("Transaction failed: {}", e);
                    transaction.rollback().await.unwrap();
                    return;
                },
            }
        }

        // update invited_by from other users
        sqlx::query!(r#"UPDATE users
            SET invited_by = (
                SELECT invited_by FROM users WHERE id = ?
            )
            WHERE invited_by = ?"#, user_id, user_id)
            .execute(&mut *transaction).await.unwrap();

        // Delete user
        sqlx::query!("DELETE FROM users WHERE id = ?", user_id)
            .execute(&mut *transaction).await.unwrap();

        transaction.commit().await.unwrap();
    }

    pub async fn set_username(
        &self,
        user_id: UserId,
        new_name: String,
    ) {
        sqlx::query!("UPDATE users SET name = ? WHERE id = ?",
            new_name, user_id).execute(&self.conn).await.unwrap();
    }

    /// Returns false if another user with the same mail exists, true on success.
    pub async fn set_email(
        &self,
        user_id: UserId,
        new_email: String,
    ) -> bool {
        let mut tx = self.conn.begin().await.unwrap();
        let existing = sqlx::query!("SELECT id FROM users WHERE email = ?", new_email)
            .fetch_optional(&mut *tx).await.unwrap();
        if existing.is_some() {
            return false;
        }
        sqlx::query!("UPDATE users SET email = ? WHERE id = ?",
            new_email, user_id).execute(&mut *tx).await.unwrap();
        tx.commit().await.unwrap();
        true
    }

    /// Whether updater_id is allowed to make changes to user_id
    pub async fn can_change(&self, updater_id: UserId, user_id: UserId) -> bool {
        match rank_users(&mut self.conn.begin().await.unwrap(), updater_id, user_id).await {
            UserRank::MissingUser | UserRank::NeitherIsAdmin | UserRank::Less => false,
            UserRank::Greater | UserRank::Equal => true,
        }
    }

    pub async fn set_is_admin(
        &self,
        user_id: UserId,
        updater_id: UserId,
        is_admin: bool,
    ) {
        let mut tx = self.conn.begin().await.unwrap();
        let user = sqlx::query!("SELECT invited_by, is_admin FROM users WHERE id = ?", user_id)
            .fetch_one(&mut *tx).await.expect("Called for non-existent user");
        if is_admin == user.is_admin { return; }

        let reduce_rank = match user.invited_by {
            None => true,
            Some(inviter) => rank_users(&mut tx, inviter, updater_id).await == UserRank::Greater
        };

        let new_inviter = if reduce_rank { Some(updater_id) } else { user.invited_by };
        sqlx::query!("UPDATE users SET is_admin = ?, invited_by = ? WHERE id = ?",
            is_admin,
            new_inviter,
            user_id).execute(&mut *tx).await.unwrap();
        tx.commit().await.unwrap();
    }

    pub async fn set_client_settings(
        &self,
        user_id: UserId,
        new_settings: serde_json::Map<String, serde_json::Value>,
    ) {
        let new_settings = serde_json::to_vec(&new_settings)
            .expect("Unable to serialize settings");
        sqlx::query!("UPDATE users SET client_settings = ? WHERE id = ?",
            new_settings, user_id).execute(&self.conn).await.unwrap();
    }

    /// Returns the user or none if the user doesn't exist or the password is wrong
    pub async fn login(&self, email: String, password: String) -> Option<UserPrivateSummary> {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};

        let user_details = sqlx::query!("SELECT id, name, password_hash, is_admin, client_settings FROM users WHERE email = ?", email)
            .fetch_optional(&self.conn).await.unwrap()?;
        let parsed_hash = PasswordHash::new(&user_details.password_hash).expect("DB corrupt");

        let argon2 = Argon2::default();
        argon2.verify_password(password.as_bytes(), &parsed_hash).ok()?;

        let client_settings = user_details.client_settings
            .map(|s| serde_json::from_slice(&s).expect("db corrupted"))
            .unwrap_or(serde_json::Map::default());
        Some(UserPrivateSummary {
            id: user_details.id,
            name: user_details.name,
            email,
            is_admin: user_details.is_admin,
            client_settings: FreeJson(client_settings),
        })
    }

    pub async fn user_from_id(&self, user_id: UserId) -> Option<UserPrivateSummary> {
        sqlx::query_as!(UserPrivateSummary,
            r#"SELECT id, name, email, is_admin, client_settings
               FROM users WHERE id = ?"#, user_id)
            .fetch_optional(&self.conn)
            .await
            .unwrap()
    }

    pub async fn project_exists(& self, project_id: ProjectId) -> bool {
        sqlx::query!("SELECT id FROM projects WHERE id = ?", project_id).fetch_optional(&self.conn).await.unwrap().is_some()
    }

    pub async fn contacts(&self, user_id: UserId) -> Vec<UserPublicSummary> {
        sqlx::query_as!(UserPublicSummary, r#"
            SELECT DISTINCT U.id, U.name
            FROM project_memberships M
            JOIN users U ON M.user_id = U.id
            WHERE M.project_id IN (
                SELECT project_id FROM project_memberships WHERE user_id = ?
            ) AND M.user_id <> ?"#,
            user_id, user_id
        )
        .fetch_all(&self.conn)
        .await
        .unwrap()
    }

    pub async fn insert_project(&self, user_id: UserId, title: String) -> Option<ProjectId> {
        if !self.settings.user_projects() {
            let is_admin = self.user_from_id(user_id).await.is_some_and(|u| u.is_admin);
            if !is_admin {
                return None;
            }
        }

        let now = EncodableDateTime::now();
        let project_id = ProjectId::random();
        if let Err(e) = sqlx::query!(
            "INSERT INTO projects (id, owner_id, title, last_modified, last_modified_by, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
                project_id, user_id, title, now, user_id, now
            ).execute(&self.conn).await {
                warn!("Projects insert failed: {e}");
                return None;
            }

        sqlx::query!("INSERT INTO project_memberships (user_id, project_id) VALUES (?, ?)", user_id, project_id)
            .execute(&self.conn).await.unwrap();
        Some(project_id)
    }

    /// Checks whether [user] owns project and returns whether a project was deleted
    pub async fn delete_project(
        &self,
        project_id: ProjectId,
        owner_id: UserId,
    ) -> DeleteProjectResult {
        info!("Deleting project {} as {}", project_id, owner_id);
        let mut transaction = self.conn.begin().await.unwrap();
        let project = sqlx::query!("SELECT owner_id FROM projects WHERE id = ?", project_id)
            .fetch_optional(&mut *transaction)
            .await.unwrap();
        let Some(project) = project else {
            return DeleteProjectResult::NotFound;
        };
        if project.owner_id != owner_id {
            return DeleteProjectResult::NotAlllowed;
        }
        sqlx::query!("DELETE FROM projects WHERE id = ?", &project_id)
            .execute(&mut *transaction)
            .await.unwrap();
        transaction.commit().await.unwrap();
        DeleteProjectResult::SuccessfullyDeleted
    }

    pub async fn list_projects(
        &self,
        user_id: UserId,
    ) -> Option<Vec<ProjectModel>> {
        sqlx::query_as!(ProjectModel, r#"SELECT
                P.id, P.title, P.owner_id, P.created_at, P.last_modified, P.last_modified_by
            FROM project_memberships M
            JOIN projects P ON M.project_id = P.id
            WHERE M.user_id = ?"#,
            user_id
        )
        .fetch_all(&self.conn)
        .await
        .ok()
    }

    pub async fn project_exists_for_user(
        &self,
        project_id: ProjectId,
        user_id: UserId,
    ) -> bool {
        sqlx::query!("SELECT user_id FROM project_memberships WHERE project_id = ? AND user_id = ?", project_id, user_id).fetch_optional(&self.conn)
            .await
            .unwrap()
            .is_some()
    }

    pub async fn get_project_for_user(
        &self,
        project_id: ProjectId,
        user_id: UserId,
    ) -> Option<ProjectModel> {
        sqlx::query!(r#"SELECT
                P.id, P.title, P.owner_id, P.created_at, P.last_modified, P.last_modified_by
            FROM project_memberships M
            JOIN projects P ON M.project_id = P.id
            WHERE M.user_id = ? AND M.project_id = ?"#,

            user_id, project_id,
        )
        .fetch_optional(&self.conn)
        .await
        .ok()?
        .map(|v| ProjectModel {
            id: v.id,
            title: v.title,
            owner_id: v.owner_id,
            created_at: EncodableDateTime::from(v.created_at),
            last_modified: EncodableDateTime::from(v.last_modified),
            last_modified_by: v.last_modified_by,
        })
    }

    pub async fn get_latest_compilation_for_user(
        &self,
        project_id: ProjectId,
        user_id: UserId,
    ) -> Option<Vec<u8>> {
        sqlx::query!(r#"
            SELECT P.latest_compilation FROM project_memberships M
            JOIN projects P ON M.project_id = P.id
            WHERE M.user_id = ? AND M.project_id = ?"#,
            user_id, project_id,
        )
        .fetch_optional(&self.conn)
        .await
        .ok()??.latest_compilation
    }


    pub async fn get_project_members(
        &self,
        project_id: ProjectId,
    ) -> Option<Vec<UserPublicSummary>> {
        sqlx::query_as!(UserPublicSummary, r#"SELECT U.id, U.name FROM project_memberships M
            JOIN users U ON M.user_id = U.id
            WHERE M.project_id = ?"#,
            project_id
        )
        .fetch_all(&self.conn)
        .await
        .ok()
    }

    pub async fn get_project_metadata_for_user(
        &self,
        project_id: ProjectId,
        user_id: UserId,
    ) -> Option<String> {
        sqlx::query!(r#"SELECT user_meta FROM project_memberships
            WHERE user_id = ? AND project_id = ?"#, user_id, project_id)
            .fetch_optional(&self.conn)
            .await
            .ok()??
            .user_meta
    }

    pub async fn set_project_metadata_for_user(
        &self,
        project_id: ProjectId,
        user_id: UserId,
        metadata: String,
    ) -> Option<bool> {
        sqlx::query!("UPDATE project_memberships SET user_meta = ? WHERE user_id = ? AND project_id = ?",
            metadata, user_id, project_id)
            .execute(&self.conn).await
            .ok().map(|r| r.rows_affected() >= 1)
    }

    /// Creates a share token for the project, if sharer has access
    pub async fn share(
        &self,
        project_id: ProjectId,
        sharer_id: UserId,
    ) -> Option<String> {
        let owner = sqlx::query!("SELECT owner_id FROM projects WHERE id = ? AND owner_id = ?", project_id, sharer_id)
            .fetch_optional(&self.conn).await.ok()?;
        if owner.is_none() {
            warn!(
                "User {} tried to share {} without permissions",
                sharer_id, project_id
            );
            return None;
        }
        debug_assert!(owner.is_some_and(|o| o.owner_id == sharer_id));
        let token = self.token_manager.add_project_join_token(project_id.clone(), sharer_id).await;
        Some(token)
    }

    pub fn preview_share(&self, token: &str) -> impl Future<Output = Option<api::types::JoinProjectPreview>> {
        self.token_manager.preview_project_join_token(token)
    }

    pub async fn accept_share(&self, token: &str, user_id: UserId) -> bool {
        use crate::token_manager::TokenUseResult;
        let (sharer, project) = match self.token_manager.try_use_project_join_token(token).await {
            TokenUseResult::NotFound => {
                info!("{} attempted to accept non-existent share: {}", user_id, token);
                return false
            },
            TokenUseResult::Expired => {
                info!("{} attempted to accept expired share: {}", user_id, token);
                return false
            },
            TokenUseResult::Valid(data) => data,
        };

        let exists = sqlx::query!("SELECT id FROM projects WHERE id = ? AND owner_id = ?", project, sharer)
            .fetch_optional(&self.conn).await.unwrap().is_some();
        if !exists {
            warn!("Token usage failed. Project {} no longer exists or is no longer owned by {}", project, sharer);
            return false;
        }

        if self.project_exists_for_user(project, user_id).await {
            return true;
        }
        sqlx::query!("INSERT INTO project_memberships (user_id, project_id) VALUES (?, ?)",
            user_id, project).execute(&self.conn).await.is_ok()
    }

    pub async fn zip(&self, project_id: ProjectId, writer: DuplexStream) {
        self.project(project_id).await.zip(writer).await
    }

    pub async fn project(&self, project_id: ProjectId) -> Arc<websocket::WebsocketProject<Manager>> {
        // TODO: write a more idiomatic version with a less ridiculous amount of cloning
        {
            let projects = self.projects.read().await;
            if let Some(project) = projects.get(&project_id) {
                return project.clone();
            }
        }
        let new_proj = websocket::WebsocketProject::load(
            self.conn.clone(),
            project_id,
            self.sandbox_manager.clone()).await;
        let new_proj = Arc::new(new_proj);
        let mut projects = self.projects.write().await;
        projects.insert(project_id.clone(), new_proj.clone());
        new_proj
    }

    pub async fn gc(&self) {
        self.projects
            .write()
            .await
            .retain(|_, v| v.can_be_disposed())
    }

    pub(crate) async fn change_title(&self, project_id: ProjectId, user_id: UserId, new_title: String) -> bool {
        let res = sqlx::query!("UPDATE projects SET title = ? WHERE id = ? AND owner_id = ?",
            new_title, project_id, user_id).execute(&self.conn).await;
        res.unwrap().rows_affected() > 0
    }

    pub(crate) async fn remove_user_from_project(&self, project_id: ProjectId, user_id: UserId) {
        let mut transaction = self.conn.begin().await.unwrap();
        match remove_user_from_project(&mut transaction, project_id, user_id).await {
            Ok(_) => transaction.commit().await,
            Err(e) => {
                error!("Transaction failed: {}", e);
                transaction.rollback().await
            },
        }.unwrap();
    }
}

async fn rank_users<'c>(
    executor: &mut sqlx::Transaction<'c, sqlx::Sqlite>,
    left_id: UserId,
    right_id: UserId,
) -> UserRank {
    let Some(left) = sqlx::query!("SELECT is_admin, invited_by FROM users WHERE id = ?", left_id)
        .fetch_optional(&mut **executor).await.unwrap() else { return UserRank::MissingUser };
    let Some(right) = sqlx::query!("SELECT is_admin, invited_by FROM users WHERE id = ?", right_id)
        .fetch_optional(&mut **executor).await.unwrap() else { return UserRank::MissingUser };
    match (left.is_admin, right.is_admin) {
        (true, true) => {}, // continue
        (false, false) => return UserRank::NeitherIsAdmin,
        (true, false) => return UserRank::Greater,
        (false, true) => return UserRank::Less,
    };

    match (left.invited_by, right.invited_by) {
        (None, None) => UserRank::Equal,
        (Some(a), Some(b)) if a == b => UserRank::Equal,
        (None, Some(_)) => UserRank::Greater,
        (Some(_), None) => UserRank::Less,
        (Some(l), Some(r)) => Box::pin(rank_users(executor, l, r)).await,
    }
}

#[derive(PartialEq, Debug)]
enum UserRank {
    MissingUser,
    NeitherIsAdmin,
    Greater,
    Less,
    Equal,
}

async fn remove_user_from_project<'c>(
    executor: &mut sqlx::Transaction<'c, sqlx::Sqlite>,
    project_id: ProjectId,
    user_id: UserId,
) -> Result<(), sqlx::Error> {
    let Some(owner) = sqlx::query!("SELECT owner_id FROM projects WHERE id = ?", project_id)
        .fetch_optional(&mut **executor).await? else {
            warn!("Attempted to remove {} from nonexistent project {}", user_id, project_id);
            return Ok(());
        };

    sqlx::query!("DELETE FROM project_memberships WHERE user_id = ? AND project_id = ?",
        user_id, project_id).execute(&mut **executor).await?;

    if owner.owner_id == user_id {
        let other = sqlx::query!("SELECT user_id FROM project_memberships WHERE project_id = ? LIMIT 1",
            project_id).fetch_optional(&mut **executor).await?;
        if let Some(other) = other {
            info!("Owner left project '{}'. Adopted by '{}'", project_id, other.user_id);
            sqlx::query!("UPDATE projects SET owner_id = ? WHERE id = ?",
                other.user_id, project_id).execute(&mut **executor).await?;
        } else {
            warn!("Didn't find a user to adopt {}. Deleting it...", project_id.to_string());
            sqlx::query!("DELETE FROM projects WHERE id = ?", project_id)
                .execute(&mut **executor).await?;
        }
    }
    Ok(())
}

// FIXME: use time directly
#[derive(Debug, PartialEq)]
pub struct EncodableDateTime(pub time::OffsetDateTime);

impl EncodableDateTime {
    pub fn now() -> EncodableDateTime {
        EncodableDateTime(time::OffsetDateTime::now_utc())
    }

}

impl From<i64> for EncodableDateTime {
    fn from(value: i64) -> Self {
        EncodableDateTime(time::OffsetDateTime::from_unix_timestamp(value).expect("EncodableDateTime from non-db value"))
    }
}

impl<'q> sqlx::Encode<'q, Sqlite> for EncodableDateTime {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let timestamp = (self.0 - time::OffsetDateTime::UNIX_EPOCH)
            .whole_seconds();

        <i64 as sqlx::Encode<'q, Sqlite>>::encode_by_ref(&timestamp, buf)
    }
}

impl<'r> sqlx::Decode<'r, Sqlite> for EncodableDateTime {
    fn decode(
        value: <Sqlite as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let inner = <i64 as sqlx::Decode<'r, Sqlite>>::decode(value)?;
        match OffsetDateTime::from_unix_timestamp(inner) {
            Ok(offset_time) => Ok(EncodableDateTime(offset_time)),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl<'q> sqlx::Type<Sqlite> for EncodableDateTime {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <i64 as sqlx::Type<Sqlite>>::type_info()
    }
}

#[derive(Clone)]
pub struct UserPrivateSummary {
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub is_admin: bool,
    pub client_settings: FreeJson,
}

#[derive(Debug, Clone, Default)]
pub struct FreeJson(pub serde_json::Map<String, serde_json::Value>);

#[derive(Debug, PartialEq)]
pub struct UserPublicSummary {
    pub id: UserId,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct ProjectModel {
    pub id: ProjectId,
    pub title: String,
    pub owner_id: UserId,
    pub created_at: EncodableDateTime,
    pub last_modified: EncodableDateTime,
    pub last_modified_by: UserId,
}

#[derive(Debug, PartialEq)]
pub enum CreateAccountResult {
    Created {
        id: UserId,
    },
    Exists,
    InvalidToken,
    ExpiredToken,
    /// Name too short, password too weak
    BadRequest,
}

impl Into<FreeJson> for Option<Vec<u8>> {
    fn into(self) -> FreeJson {
        let Some(data) = self else { return Default::default() };
        FreeJson(serde_json::from_slice(&data).unwrap())
    }
}

#[derive(Debug)]
pub enum DeleteProjectResult {
    NotFound,
    NotAlllowed,
    SuccessfullyDeleted,
}

#[cfg(all(test, not(feature = "fireframe")))]
pub mod tests {
    use std::assert_matches;

use crate::app::{self, App, CreateAccountResult, DeleteProjectResult, ProjectId};

    #[tokio::test]
    async fn can_create_accounts() {
        let app = App::new("sqlite::memory:").await.unwrap();

        let token = app.add_registration_token(None).await;
        let res = app
            .register(
                token,
                "foobar".into(),
                "foo@bar.de".into(),
                "foobar!1sadf".into(),
            )
            .await;
        assert!(
            matches!(res, CreateAccountResult::Created { .. }),
            "{:?}",
            res,
        );
    }

    #[tokio::test]
    async fn can_login() {
        let app = App::new("sqlite::memory:").await.unwrap();

        let token = app.add_registration_token(None).await;
        let res = app
            .register(
                token,
                "foobar".into(),
                "foo@bar.de".into(),
                "foobar!1sadf".into(),
            )
            .await;
        assert!(
            matches!(res, CreateAccountResult::Created { .. }),
            "{:?}",
            res,
        );

        let res = app.login("foo@bar.de".into(), "foobar!1sadf".into()).await;
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name, "foobar");
        assert_eq!(res.email, "foo@bar.de");
    }

    #[tokio::test]
    async fn cant_create_accounts_without_token() {
        let app = App::new("sqlite::memory:").await.unwrap();

        let res = app
            .register(
                "not a token".into(),
                "foobar".into(),
                "foo@bar.de".into(),
                "foobar!1sadf".into(),
            )
            .await;
        assert_eq!(res, CreateAccountResult::InvalidToken);
    }

    #[tokio::test]
    async fn doesnt_generate_same_token_twice() {
        let app = App::new("sqlite::memory:").await.unwrap();

        let token1 = app.add_registration_token(None).await;
        let token2 = app.add_registration_token(None).await;
        assert_ne!(token1, token2);
    }

    #[tokio::test]
    async fn cant_login_with_wrong_password() {
        let app = App::new("sqlite::memory:").await.unwrap();

        let token = app.add_registration_token(None).await;
        let res = app
            .register(
                token,
                "foobar".into(),
                "foo@bar.de".into(),
                "foobar!1sadf".into(),
            )
            .await;
        assert!(
            matches!(res, CreateAccountResult::Created { .. }),
            "{:?}",
            res,
        );

        let res = app.login("foo@bar.de".into(), "wrong pw".into()).await;
        assert!(res.is_none());
    }

    #[tokio::test]
    async fn can_create_and_list_projects() {
        let app = App::new("sqlite::memory:").await.unwrap();

        let token = app.add_registration_token(None).await;
        let res = app
            .register(
                token,
                "foobar".into(),
                "foo@bar.de".into(),
                "foobar!1sadf".into(),
            )
            .await;
        let user_id = match res {
            CreateAccountResult::Created { id } => id,
            _ => panic!("Failed to create account"),
        };

        let projects = app.list_projects(user_id).await;
        assert!(projects.is_some_and(|projects| projects.is_empty()));

        let foo_project = app
            .insert_project(user_id, "Test Title".into())
            .await.unwrap();

        let projects = app.list_projects(user_id).await.unwrap();
        assert_eq!(projects.len(), 1);

        let bar_project = app
            .insert_project(user_id, "Test Title".into())
            .await.unwrap();
        assert_ne!(foo_project, bar_project);

        let projects = app.list_projects(user_id).await.unwrap();
        assert_eq!(projects.len(), 2);

        let projects = app.list_projects(user_id).await.unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[tokio::test]
    async fn admins_can_issue_tokens() {
        let app = App::new("sqlite::memory:").await.unwrap();
        assert_eq!(app.user_count().await, 0);

        let token = app.add_registration_token(None).await;
        let CreateAccountResult::Created { id: admin_id } = app
            .register(
                token,
                "admin".into(),
                "admin@admin.com".into(),
                "adminadmin".into(),
            )
            .await
        else {
            panic!()
        };

        let admin = app.user_from_id(admin_id).await.unwrap();
        assert!(admin.is_admin);
        assert_eq!(app.user_count().await, 1);

        let token = app.create_invite(admin).await.unwrap();
        let CreateAccountResult::Created { id } = app
            .register(
                token,
                "foobar".into(),
                "foo@bar.de".into(),
                "foobar!1sadf".into(),
            )
            .await
        else {
            panic!()
        };

        let user = app.user_from_id(id).await.unwrap();
        assert!(!user.is_admin);
        //assert_eq!(user.invited_by.unwrap(), admin_id);
        assert_eq!(app.user_count().await, 2);
    }

    #[tokio::test]
    async fn open_signups_setting_works() {
        let app = App::new("sqlite::memory:").await.unwrap();
        assert_eq!(app.settings.open_sign_ups(), false);
        assert_eq!(app.user_count().await, 0);

        let res = app.register(
            "no valid token".into(),
            "user".into(),
            "user@user.com".into(),
            "useruser".into(),
        ).await;
        assert_matches!(res, CreateAccountResult::InvalidToken);
        assert_eq!(app.user_count().await, 0);

        app.settings.set_open_sign_ups(true).await.unwrap();

        let res = app.register(
            "no valid token".into(),
            "user".into(),
            "user@user.com".into(),
            "useruser".into(),
        ).await;
        assert_matches!(res, CreateAccountResult::Created { id: _ });
        assert_eq!(app.user_count().await, 1);
    }

    #[tokio::test]
    async fn users_can_create_projects() {
        let (app, _, user) = sample_app().await;

        assert_eq!(app.project_count().await, 0);
        assert_eq!(app.list_projects(user.id).await.unwrap().len(), 0);

        let pid = app.insert_project(user.id, "Cool Project".into())
            .await.unwrap();

        assert_eq!(app.project_count().await, 1);
        assert_eq!(app.list_projects(user.id).await.unwrap().len(), 1);
        let projects = app.list_projects(user.id).await.unwrap();
        let project = projects.first().unwrap();
        assert_eq!(project.id, pid);
        assert_eq!(project.title, "Cool Project");
        assert_eq!(project.owner_id, user.id);

        assert!(app.project_exists(project.id).await);
        assert!(app.project_exists_for_user(project.id, user.id).await);
        assert_eq!(
            &app.get_project_for_user(project.id, user.id)
                .await
                .unwrap(),
            project
        );
    }

    #[tokio::test]
    async fn user_projects_setting_works() {
        let (app, admin, user) = sample_app().await;
        app.settings.set_user_projects(false).await.unwrap();

        assert_eq!(app.project_count().await, 0);
        let pid = app.insert_project(user.id, "Cool Project".into())
            .await;
        assert!(pid.is_none());
        assert_eq!(app.project_count().await, 0);

        app.insert_project(admin.id, "Cool Project".into())
            .await.expect("Project created");
        assert_eq!(app.project_count().await, 1);
    }

    #[tokio::test]
    async fn users_can_share_projects_with_others() {
        let (app, admin, user) = sample_app().await;

        assert_eq!(app.list_projects(admin.id).await.unwrap().len(), 0);
        assert_eq!(app.list_projects(user.id).await.unwrap().len(), 0);

        let pid = app.insert_project(user.id, "Cool Project".into())
            .await.unwrap();

        assert_eq!(app.list_projects(admin.id).await.unwrap().len(), 0);
        assert_eq!(app.list_projects(user.id).await.unwrap().len(), 1);

        let share = app.share(pid, user.id).await.unwrap();
        assert_eq!(app.list_projects(admin.id).await.unwrap().len(), 0);

        app.accept_share(&share, admin.id).await;
        assert_eq!(app.list_projects(admin.id).await.unwrap().len(), 1);
        assert_eq!(app.list_projects(user.id).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn shared_project_information_is_correct() {
        let (app, _, user) = sample_app().await;
        let pid = app.insert_project(user.id, "Cool Project".into())
            .await.unwrap();
        let project = app.get_project_for_user(pid, user.id).await.unwrap();

        let share = app.share(pid, user.id).await.unwrap();
        let info = app.preview_share(&share).await.unwrap();
        assert_eq!(info.created_at, project.created_at.0);
        assert_eq!(info.updated_at, project.last_modified.0);
        assert_eq!(info.id, project.id);
        assert_eq!(info.title, project.title);
        assert_eq!(info.owner.id, user.id);
        assert_eq!(info.owner.username, user.name);
    }

    #[tokio::test]
    async fn cant_share_foreign_projects() {
        let (app, admin, user) = sample_app().await;
        let pid = app.insert_project(user.id, "Cool Project".into())
            .await.unwrap();
        let share = app.share(pid, admin.id).await;
        assert!(share.is_none(), "Can't share when not member");

        let share = app.share(pid, user.id).await.unwrap();
        app.accept_share(&share, admin.id).await;

        let share = app.share(pid, admin.id).await;
        assert!(share.is_none(), "Can't share when not owner");
    }

    #[tokio::test]
    async fn users_can_resolve_their_contacts() {
        let (app, admin, user) = sample_app().await;
        let token = app.create_invite(admin.clone()).await;
        let CreateAccountResult::Created { id: user2 } = app.register(
            token.unwrap(),
            "user2".into(),
            "user2@dummy.com".into(),
            "dummydummy".into()).await else {
            panic!()
        };
        let token = app.create_invite(admin.clone()).await;
        let CreateAccountResult::Created { id: user3 } = app.register(
            token.unwrap(),
            "user3".into(),
            "user3@dummy.com".into(),
            "dummydummy".into()).await else {
            panic!()
        };

        assert_eq!(app.contacts(user.id).await.len(), 0);
        let p1 = app.insert_project(user.id, "P 1".into())
            .await.unwrap();
        let share = app.share(p1, user.id).await.unwrap();
        dbg!(app.contacts(user.id).await);
        assert_eq!(app.contacts(user.id).await.len(), 0);
        app.accept_share(&share, user2).await;
        assert_eq!(app.contacts(user.id).await.first().unwrap().id, user2);
        assert_eq!(app.contacts(user.id).await.first().unwrap().name, "user2");
        assert_eq!(app.contacts(user2).await.first().unwrap().id, user.id);

        let p2 = app.insert_project(user.id, "P 2".into())
            .await.unwrap();
        let share = app.share(p2, user.id).await.unwrap();
        app.accept_share(&share, user2).await;
        let share = app.share(p2, user.id).await.unwrap();
        app.accept_share(&share, user3).await;
        assert_eq!(app.contacts(user.id).await.len(), 2);
        assert!(app.contacts(user.id).await.contains(&app::UserPublicSummary { id: user2, name: "user2".into() }));
        assert!(app.contacts(user.id).await.contains(&app::UserPublicSummary { id: user3, name: "user3".into() }));
        assert!(!app.contacts(user.id).await.contains(&app::UserPublicSummary { id: admin.id, name: admin.name }));
        assert_eq!(app.contacts(user2).await.len(), 2);
    }

    #[tokio::test]
    async fn deleting_project_deletes_referencing_fields() {
        let (app, _, user) = sample_app().await;

        assert_eq!(app.get_project_members(ProjectId::random()).await.unwrap().len(), 0);
        let pid = app.insert_project(user.id, "Foo Project".into()).await.unwrap();
        assert_eq!(app.get_project_members(pid).await.unwrap().len(), 1);
        assert_matches!(app.delete_project(pid, user.id).await, DeleteProjectResult::SuccessfullyDeleted);
        assert_eq!(app.get_project_members(pid).await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn unable_to_delete_others_project() {
        let (app, admin, user) = sample_app().await;

        let pid = app.insert_project(user.id, "Foo Project".into()).await.unwrap();
        assert_matches!(app.delete_project(pid, admin.id).await, DeleteProjectResult::NotAlllowed);
    }

    #[tokio::test]
    async fn can_update_user_data() {
        let app = App::new("sqlite::memory:").await.unwrap();

        let token = app.add_registration_token(None).await;
        let CreateAccountResult::Created { id } = app.register(
            token,
            "initialName".into(),
            "initialMail".into(),
            "foobar123x".into(),
        ).await else { panic!() };
        let user = app.login("initialMail".into(), "foobar123x".into()).await.unwrap();
        assert_eq!(user.id, id);
        assert_eq!(user.name, "initialName");
        assert_eq!(user.email, "initialMail");
        assert_eq!(user.client_settings.0, serde_json::Map::new());

        app.set_email(user.id, "new_email".into()).await;
        assert!(app.login("initialMail".into(), "foobar123x".into()).await.is_none());
        let user = app.login("new_email".into(), "foobar123x".into()).await.unwrap();
        assert_eq!(user.email, "new_email");

        app.set_username(user.id, "new_name".into()).await;
        let user = app.login("new_email".into(), "foobar123x".into()).await.unwrap();
        assert_eq!(user.name, "new_name");

        app.set_client_settings(user.id, serde_json::json!({"x": 42}).as_object().unwrap().clone()).await;
        let user = app.login("new_email".into(), "foobar123x".into()).await.unwrap();
        assert_eq!(user.client_settings.0, serde_json::json!({"x": 42}).as_object().unwrap().clone());
    }

    #[tokio::test]
    async fn only_owner_can_change_project_title() {
        let (app, admin, user) = sample_app().await;
        let pid = app.insert_project(user.id, "Title 1".into())
            .await.unwrap();
        let share = app.share(pid, user.id).await.unwrap();
        app.accept_share(&share, admin.id).await;

        assert_eq!(app.get_project_for_user(pid, user.id).await.unwrap().title, "Title 1");
        assert!(!app.change_title(pid, admin.id, "new_title".into()).await);
        assert_eq!(app.get_project_for_user(pid, user.id).await.unwrap().title, "Title 1");
        assert!(app.change_title(pid, user.id, "Title 2".into()).await);
        assert_eq!(app.get_project_for_user(pid, user.id).await.unwrap().title, "Title 2");
    }

    #[tokio::test]
    async fn can_change_works() {
        let (app, admin, user) = sample_app().await;
        assert!(app.can_change(admin.id, user.id).await);
        assert!(!app.can_change(user.id, admin.id).await);
        assert!(!app.can_change(user.id, user.id).await);
        assert!(app.can_change(admin.id, admin.id).await);
    }

    /// Sets up app with one admin and one dummy user.
    pub async fn sample_app() -> (App, app::UserPrivateSummary, app::UserPrivateSummary) {
        let app = App::new("sqlite::memory:").await.unwrap();

        let token = app.add_registration_token(None).await;
        let register_result = app
            .register(
                token,
                "admin".into(),
                "admin@admin.com".into(),
                "adminadmin".into(),
            )
            .await;
        let admin_id = match register_result {
            CreateAccountResult::Created { id, } => id,
            value => panic!("Didn't expect: {value:#?}")
        };
        let admin = app.user_from_id(admin_id).await.unwrap();

        let token = app.create_invite(admin.clone()).await;
        let CreateAccountResult::Created { id: dummy_id } = app
            .register(
                token.unwrap(),
                "dummy".into(),
                "dummy@dummy.com".into(),
                "dummydummy".into(),
            )
            .await
        else {
            panic!()
        };
        let dummy = app.user_from_id(dummy_id).await.unwrap();

        (app, admin, dummy)
    }
}
