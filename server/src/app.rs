use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "fireframe")]
use crate::project::sandbox_manger;
use crate::token_manager::TokenManager;
use crate::{websocket};
use sqlx::{Database, Sqlite, SqlitePool};
use time::OffsetDateTime;
use tokio::io::DuplexStream;
use tokio::sync::{RwLock};
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct App {
    conn: SqlitePool,
    token_manager: Arc<TokenManager>,
    projects: Arc<RwLock<HashMap<ProjectID, websocket::WebsocketProject>>>,
    #[cfg(feature = "fireframe")]
    sandbox_manager: Arc<sandbox_manger::SandboxManager>,
}

impl App {
    pub async fn new(
        db_path: &str,
        #[cfg(feature = "fireframe")]
        sandbox_manager: Arc<sandbox_manger::SandboxManager>) -> Result<Self, String>
    {
        let conn = Self::initialize_db(db_path).await?;
        Ok(Self {
            token_manager: Arc::new(TokenManager::new(conn.clone())),
            #[cfg(feature = "fireframe")]
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

    pub fn add_registration_token(&self, issuer_id: Option<UserID>) -> impl Future<Output = String> {
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

        let issuer = match self.token_manager.try_use_registration_token(&registration_token).await {
            TokenUseResult::NotFound => return CreateAccountResult::InvalidToken,
            TokenUseResult::Expired => return CreateAccountResult::ExpiredToken,
            TokenUseResult::Valid(issuer) => issuer,
        };

        // Save account
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        let argon2 = Argon2::default();
        let Ok(password_hash) = argon2.hash_password(password.as_bytes(), &salt) else {
            return CreateAccountResult::BadRequest;
        };
        let password_hash = password_hash.serialize().to_string();
        let is_admin = issuer.is_none();
        sqlx::query!(r#"INSERT INTO users (name, email, password_hash, is_admin, invited_by)
            VALUES (?, ?, ?, ?, ?)"#,
            name,
            email,
            password_hash,
            is_admin,
            issuer,
        ).execute(&self.conn).await.unwrap();
        let new_user = sqlx::query!("SELECT id FROM users WHERE email = ?", email).fetch_one(&self.conn).await.unwrap();

        CreateAccountResult::Created { id: UserID(new_user.id) }
    }

    /// Returns the user or none if the user doesn't exist or the password is wrong
    pub async fn login(&self, email: String, password: String) -> Option<UserPrivateSummary> {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};

        let user_details = sqlx::query!("SELECT id, name, password_hash, is_admin FROM users WHERE email = ?", email)
            .fetch_optional(&self.conn).await.unwrap()?;
        let parsed_hash = PasswordHash::new(&user_details.password_hash).expect("DB corrupt");

        let argon2 = Argon2::default();
        argon2.verify_password(password.as_bytes(), &parsed_hash).ok()?;

        Some(UserPrivateSummary {
            id: UserID(user_details.id),
            name: user_details.name,
            email,
            is_admin: user_details.is_admin,
        })
    }

    pub async fn user_from_id(&self, user_id: UserID) -> Option<UserPrivateSummary> {
        sqlx::query_as!(UserPrivateSummary, "SELECT id, name, email, is_admin FROM users WHERE id = ?", user_id)
            .fetch_optional(&self.conn)
            .await
            .unwrap()
    }

    pub async fn project_exists(& self, project_id: &ProjectID) -> bool {
        sqlx::query!("SELECT id FROM projects WHERE id = ?", project_id).fetch_optional(&self.conn).await.unwrap().is_some()
    }

    pub async fn contacts(&self, user_id: UserID) -> Vec<UserPublicSummary> {
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

    pub async fn insert_project(&self, project_id: &ProjectID, user_id: UserID, title: String) -> bool {
        let now = EncodableDateTime::now();
        if let Err(e) = sqlx::query!(
            "INSERT INTO projects (id, owner_id, title, last_modified, last_modified_by, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
                project_id, user_id, title, now, user_id, now
            ).execute(&self.conn).await {
                warn!("Projects insert failed: {e}");
                return false;
            }

        sqlx::query!("INSERT INTO project_memberships (user_id, project_id) VALUES (?, ?)", user_id, project_id)
            .execute(&self.conn).await.is_ok()
    }

    /// Checks whether [user] owns project and returns whether a project was deleted
    pub async fn delete_project(
        &self,
        project_id: &ProjectID,
        owner_id: UserID,
    ) -> bool {
        sqlx::query!("DELETE FROM projects WHERE id = ? AND owner_id = ?", project_id, owner_id)
            .execute(&self.conn)
            .await
            .is_ok_and(|res| res.rows_affected() >= 1)
    }

    pub async fn list_projects(
        &self,
        user_id: UserID,
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
        project_id: &ProjectID,
        user_id: UserID,
    ) -> bool {
        sqlx::query!("SELECT user_id FROM project_memberships WHERE project_id = ? AND user_id = ?", project_id, user_id).fetch_optional(&self.conn)
            .await
            .unwrap()
            .is_some()
    }

    #[cfg(test)]
    pub async fn get_project_for_user(
        &self,
        project_id: &ProjectID,
        user_id: UserID,
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
            id: ProjectID(v.id),
            title: v.title,
            owner_id: UserID(v.owner_id),
            created_at: EncodableDateTime::from(v.created_at),
            last_modified: EncodableDateTime::from(v.last_modified),
            last_modified_by: UserID(v.last_modified_by),
        })
    }

    pub async fn get_latest_compilation_for_user(
        &self,
        project_id: &ProjectID,
        user_id: UserID,
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
        project_id: &ProjectID,
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
        project_id: &ProjectID,
        user_id: UserID,
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
        project_id: &ProjectID,
        user_id: UserID,
        metadata: String,
    ) -> Option<bool> {
        sqlx::query!("UPDATE project_memberships SET user_meta = ? WHERE user_id = ? AND project_id = ?",
            metadata, user_id, project_id)
            .execute(&self.conn).await
            .ok().map(|r| r.rows_affected() > 1)
    }

    /// Creates a share token for the project, if sharer has access
    pub async fn share(
        &self,
        project_id: &ProjectID,
        sharer_id: UserID,
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
        debug_assert!(owner.is_some_and(|o| o.owner_id == sharer_id.0));
        let token = self.token_manager.add_project_join_token(project_id.clone(), sharer_id).await;
        Some(token)
    }

    pub async fn accept_share(&self, token: &str, user_id: UserID) -> bool {
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

        let exists = sqlx::query!("SELECT id FROM projects WHERE id = ? AND owner_id = ?", project.0, sharer)
            .fetch_optional(&self.conn).await.unwrap().is_some();
        if !exists {
            warn!("Token usage failed. Project {} no longer exists or is no longer owned by {}", project, sharer);
            return false;
        }

        if self.project_exists_for_user(&project, user_id).await {
            return true;
        }
        sqlx::query!("INSERT INTO project_memberships (user_id, project_id) VALUES (?, ?)",
            user_id, project.0).execute(&self.conn).await.is_ok()
    }

    pub async fn zip(&self, project_id: &ProjectID, writer: DuplexStream) {
        self.project(project_id).await.zip(writer).await
    }

    pub async fn project(&self, project_id: &ProjectID) -> websocket::WebsocketProject {
        // TODO: write a more idiomatic version with a less ridiculous amount of cloning
        {
            let projects = self.projects.read().await;
            if let Some(project) = projects.get(&project_id) {
                return project.clone();
            }
        }
        let new_proj = websocket::WebsocketProject::load(
            self.conn.clone(),
            project_id.to_string(),
            #[cfg(feature = "fireframe")]
            self.sandbox_manager.clone()).await;
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
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UserID(pub i64);

impl Display for UserID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <i64 as Display>::fmt(&self.0, f)
    }
}

impl<'q> sqlx::Encode<'q, Sqlite> for UserID {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i64 as sqlx::Encode<'q, Sqlite>>::encode_by_ref(&self.0, buf)
    }
}

impl sqlx::Type<Sqlite> for UserID {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <i64 as sqlx::Type<Sqlite>>::type_info()
    }
}

impl From<i64> for UserID {
    fn from(value: i64) -> Self {
        UserID(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectID(pub String);

impl Display for ProjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'q> sqlx::Encode<'q, Sqlite> for &'q ProjectID {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <String as sqlx::Encode<'q, Sqlite>>::encode_by_ref(&self.0, buf)
    }
}

impl<'q> sqlx::Type<Sqlite> for ProjectID {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }
}

impl Into<ProjectID> for String {
    fn into(self) -> ProjectID {
        ProjectID(self)
    }
}

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
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
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
    pub id: UserID,
    pub name: String,
    pub email: String,
    pub is_admin: bool,
}

#[derive(Debug, PartialEq)]
pub struct UserPublicSummary {
    pub id: UserID,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct ProjectModel {
    pub id: ProjectID,
    pub title: String,
    pub owner_id: UserID,
    pub created_at: EncodableDateTime,
    pub last_modified: EncodableDateTime,
    pub last_modified_by: UserID,
}

#[derive(Debug, PartialEq)]
pub enum CreateAccountResult {
    Created {
        id: UserID,
    },
    Exists,
    InvalidToken,
    ExpiredToken,
    /// Name too short, password too weak
    BadRequest,
}

#[cfg(all(test, not(feature = "fireframe")))]
pub mod tests {
    use crate::app::{self, App, CreateAccountResult, ProjectID};

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

        let res = app
            .insert_project(&ProjectID("test_project".into()), user_id, "Test Title".into())
            .await;
        assert!(res, "Project creation failed");

        let projects = app.list_projects(user_id).await.unwrap();
        assert_eq!(projects.len(), 1);

        let res = app
            .insert_project(&ProjectID("test_project2".into()), user_id, "Test Title".into())
            .await;
        assert!(res, "Project creation failed");

        let projects = app.list_projects(user_id).await.unwrap();
        assert_eq!(projects.len(), 2);

        let res = app
            .insert_project(&ProjectID("test_project".into()), user_id, "Test Title".into())
            .await;
        assert!(!res, "Created duplicate projects");

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
    async fn users_can_create_projects() {
        let (app, _, user) = sample_app().await;

        assert_eq!(app.project_count().await, 0);
        assert_eq!(app.list_projects(user.id).await.unwrap().len(), 0);

        let project = app
            .insert_project(&ProjectID("cool_project".into()), user.id, "Cool Project".into())
            .await;
        assert!(project);

        assert_eq!(app.project_count().await, 1);
        assert_eq!(app.list_projects(user.id).await.unwrap().len(), 1);
        let projects = app.list_projects(user.id).await.unwrap();
        let project = projects.first().unwrap();
        assert_eq!(project.id.0, "cool_project");
        assert_eq!(project.title, "Cool Project");
        assert_eq!(project.owner_id, user.id);

        assert!(app.project_exists(&project.id).await);
        assert!(app.project_exists_for_user(&project.id, user.id).await);
        assert_eq!(
            &app.get_project_for_user(&project.id, user.id)
                .await
                .unwrap(),
            project
        );
    }

    #[tokio::test]
    async fn users_can_share_projects_with_others() {
        let (app, admin, user) = sample_app().await;

        assert_eq!(app.list_projects(admin.id).await.unwrap().len(), 0);
        assert_eq!(app.list_projects(user.id).await.unwrap().len(), 0);

        app.insert_project(&ProjectID("cool_project".into()), user.id, "Cool Project".into())
            .await;

        assert_eq!(app.list_projects(admin.id).await.unwrap().len(), 0);
        assert_eq!(app.list_projects(user.id).await.unwrap().len(), 1);

        let share = app.share(&ProjectID("cool_project".into()), user.id).await.unwrap();
        assert_eq!(app.list_projects(admin.id).await.unwrap().len(), 0);

        app.accept_share(&share, admin.id).await;
        assert_eq!(app.list_projects(admin.id).await.unwrap().len(), 1);
        assert_eq!(app.list_projects(user.id).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn cant_share_foreign_projects() {
        let (app, admin, user) = sample_app().await;
        app.insert_project(&ProjectID("cool_project".into()), user.id, "Cool Project".into())
            .await;
        let share = app.share(&ProjectID("cool_project".into()), admin.id).await;
        assert!(share.is_none(), "Can't share when not member");

        let share = app.share(&ProjectID("cool_project".into()), user.id).await.unwrap();
        app.accept_share(&share, admin.id).await;

        let share = app.share(&ProjectID("cool_project".into()), admin.id).await;
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
        app.insert_project(&ProjectID("p1".into()), user.id, "P 1".into())
            .await;
        let share = app.share(&ProjectID("p1".into()), user.id).await.unwrap();
        dbg!(app.contacts(user.id).await);
        assert_eq!(app.contacts(user.id).await.len(), 0);
        app.accept_share(&share, user2).await;
        assert_eq!(app.contacts(user.id).await.first().unwrap().id, user2);
        assert_eq!(app.contacts(user.id).await.first().unwrap().name, "user2");
        assert_eq!(app.contacts(user2).await.first().unwrap().id, user.id);

        app.insert_project(&ProjectID("p2".into()), user.id, "P 2".into())
            .await;
        let share = app.share(&ProjectID("p2".into()), user.id).await.unwrap();
        app.accept_share(&share, user2).await;
        let share = app.share(&ProjectID("p2".into()), user.id).await.unwrap();
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
        let pid = ProjectID("foobar".into());

        assert_eq!(app.get_project_members(&pid).await.unwrap().len(), 0);
        app.insert_project(&pid, user.id, "Foo Project".into()).await;
        assert_eq!(app.get_project_members(&pid).await.unwrap().len(), 1);
        assert_eq!(app.delete_project(&pid, user.id).await, true);
        assert_eq!(app.get_project_members(&pid).await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn unable_to_delete_others_project() {
        let (app, admin, user) = sample_app().await;
        let pid = ProjectID("foobar".into());

        app.insert_project(&pid, user.id, "Foo Project".into()).await;
        let res = app.delete_project(&pid, admin.id).await;
        assert!(!res);
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
