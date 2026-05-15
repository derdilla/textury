use std::collections::HashMap;
use std::sync::Arc;

use crate::{model, websocket};
use rand::distr::SampleString;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, ModelTrait,
    PaginatorTrait, SelectExt,
};
use sea_orm::{ColumnTrait, IntoActiveModel, QueryFilter};
use time::ext::NumericalDuration;
use tokio::io::DuplexStream;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

#[derive(Clone, Default)]
pub struct App {
    conn: DatabaseConnection,
    /// Map from share_id (long random) to user_id
    shares: Arc<RwLock<HashMap<String, String>>>,
    projects: Arc<RwLock<HashMap<String, websocket::WebsocketProject>>>,
}

impl App {
    pub async fn new(conn: DatabaseConnection) -> Self {
        let obj = Self {
            conn,
            ..Default::default()
        };
        obj.setup_schema().await;
        obj
    }

    async fn setup_schema(&self) {
        let res = self
            .conn
            .get_schema_builder()
            .register(model::user::Entity)
            .register(model::project::Entity)
            .register(model::project_memberships::Entity)
            .register(model::registration_tokens::Entity)
            .register(model::asset::Entity)
            .apply(&self.conn)
            .await;
        if let Err(res) = res {
            warn!("Failed setting up db tables: {}", res)
        }
    }

    pub async fn user_count(&self) -> u64 {
        model::user::Entity::find()
            .count(&self.conn)
            .await
            .unwrap_or(0)
    }

    #[cfg(test)]
    pub async fn project_count(&self) -> u64 {
        model::project::Entity::find()
            .count(&self.conn)
            .await
            .unwrap_or(0)
    }

    /// Create a registration token, if [inviter] is administrator.
    pub async fn create_invite(&self, inviter: model::user::Model) -> Option<String> {
        if !inviter.is_admin {
            return None;
        }

        Some(self.add_registration_token(Some(inviter.id)).await)
    }

    pub async fn add_registration_token(&self, inviter_id: Option<i32>) -> String {
        let token = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 20);
        let expires = time::OffsetDateTime::now_utc();
        let expires = expires.checked_add(15.minutes()).unwrap();
        model::registration_tokens::ActiveModel {
            token: Set(token.to_string()),
            expires: Set(expires),
            issuer_id: Set(inviter_id),
        }
        .insert(&self.conn)
        .await
        .unwrap();

        token.to_string()
    }

    pub async fn register(
        &self,
        registration_token: String,
        name: String,
        email: String,
        password: String,
    ) -> CreateAccountResult {
        use argon2::{Argon2, PasswordHasher, password_hash::SaltString};

        if name.len() < 4 || password.len() < 8 {
            return CreateAccountResult::BadRequest;
        }

        // Check for conflicts
        if model::user::Entity::find_by_email(&email)
            .exists(&self.conn)
            .await
            .unwrap()
        {
            return CreateAccountResult::Exists;
        }

        // Validate registration token
        let Ok(Some(token)) =
            model::registration_tokens::Entity::find_by_token(&registration_token)
                .one(&self.conn)
                .await
        else {
            return CreateAccountResult::InvalidToken;
        };
        if token.expires < time::OffsetDateTime::now_utc() {
            token.delete(&self.conn).await.unwrap();
            return CreateAccountResult::ExpiredToken;
        }
        let issuer = token.issuer_id;
        token.delete(&self.conn).await.unwrap();

        // Save account
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        let argon2 = Argon2::default();
        let Ok(password_hash) = argon2.hash_password(password.as_bytes(), &salt) else {
            return CreateAccountResult::BadRequest;
        };
        let new_user = model::user::ActiveModel {
            name: Set(name),
            email: Set(email),
            password_hash: Set(password_hash.to_string()),
            is_admin: Set(issuer.is_none()),
            invited_by: Set(issuer),
            ..Default::default()
        };
        let new_user = new_user.insert(&self.conn).await.unwrap();

        CreateAccountResult::Created { id: new_user.id }
    }

    /// Returns the user or none if the user doesn't exist or the password is wrong
    pub async fn login(&self, email: String, password: String) -> Option<model::user::Model> {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};

        let user_details = model::user::Entity::find_by_email(&email)
            .one(&self.conn)
            .await
            .unwrap()?;
        let parsed_hash = PasswordHash::new(&user_details.password_hash).ok()?;

        let argon2 = Argon2::default();
        argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .ok()?;

        Some(user_details)
    }

    pub async fn user_from_id(&self, user_id: i32) -> Option<model::user::Model> {
        let user = model::user::Entity::find_by_id(user_id)
            .one(&self.conn)
            .await;

        let Ok(user) = user else {
            return None;
        };

        user
    }

    pub async fn project_exists(&self, project_id: &str) -> bool {
        model::project::Entity::find_by_id(project_id)
            .exists(&self.conn)
            .await
            .unwrap()
    }

    pub async fn insert_project(&self, project_id: String, user_id: i32, title: String) -> bool {
        use sea_orm::Set;

        let now = time::OffsetDateTime::now_utc();
        let new_project = model::project::ActiveModel {
            id: Set(project_id.clone()),
            owner_id: Set(user_id),
            title: Set(title),
            last_modified: Set(time::PrimitiveDateTime::new(now.date(), now.time())),
            ..Default::default()
        };
        if new_project.insert(&self.conn).await.is_err() {
            return false;
        }

        model::project_memberships::ActiveModel {
            user_id: Set(user_id),
            project_id: Set(project_id.clone()),
            ..Default::default()
        }
        .insert(&self.conn)
        .await
        .is_ok()
    }

    /// Checks whether [user] owns project and returns whether a project was deleted
    pub async fn delete_project(
        &self,
        project_id: &str,
        user: &model::user::Model,
    ) -> Option<bool> {
        let project = model::project::Entity::find_by_id(project_id)
            .one(&self.conn)
            .await
            .ok()??;
        if project.owner_id != user.id {
            warn!(
                "Unauthorized user {} tried to delete {}",
                user.email, project.id
            );
            return None;
        }

        // TODO: close anything open

        match project.cascade_delete(&self.conn).await {
            Ok(res) => Some(res.rows_affected >= 1),
            Err(e) => {
                error!("Failed deleting: {}", e);
                None
            }
        }
    }

    pub async fn list_projects(
        &self,
        from: &model::user::Model,
    ) -> Option<Vec<model::project::Model>> {
        from.find_related(model::project::Entity)
            .all(&self.conn)
            .await
            .ok()
    }

    pub async fn project_exists_for_user(
        &self,
        project_id: &str,
        user: &model::user::Model,
    ) -> bool {
        user.find_related(model::project::Entity)
            .filter(model::project::Column::Id.eq(project_id))
            .exists(&self.conn)
            .await
            .unwrap()
    }

    pub async fn get_project_for_user(
        &self,
        project_id: &str,
        user: &model::user::Model,
    ) -> Option<model::project::Model> {
        user.find_related(model::project::Entity)
            .filter(model::project::Column::Id.eq(project_id))
            .one(&self.conn)
            .await
            .ok()?
    }

    pub async fn get_project_metadata_for_user(
        &self,
        project_id: &str,
        user_id: i32,
    ) -> Option<serde_json::Value> {
        let model = model::project_memberships::Entity::find()
            .filter(model::project_memberships::Column::ProjectId.eq(project_id))
            .filter(model::project_memberships::Column::UserId.eq(user_id))
            .one(&self.conn)
            .await;
        model.ok()??.user_meta
    }

    pub async fn set_project_metadata_for_user(
        &self,
        project_id: &str,
        user_id: i32,
        metadata: serde_json::Value,
    ) -> Option<bool> {
        let model = model::project_memberships::Entity::find()
            .filter(model::project_memberships::Column::ProjectId.eq(project_id))
            .filter(model::project_memberships::Column::UserId.eq(user_id))
            .one(&self.conn)
            .await;
        if let Ok(Some(model)) = model {
            let mut model = model.into_active_model();
            model.user_meta = Set(Some(metadata));
            let result = model.update(&self.conn).await;
            return Some(result.is_ok());
        }
        None
    }

    /// Creates a share token for the project, if sharer has access
    pub async fn share(&self, project_id: &str, sharer: &model::user::Model) -> Option<String> {
        let Some(proj) = self.get_project_for_user(project_id, sharer).await else {
            debug!("No project found for {}", project_id);
            return None;
        };

        if proj.owner_id != sharer.id {
            warn!(
                "User {} tried to share {} without permissions",
                sharer.name, proj.id
            );
            return None;
        }

        let key = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 40);
        let res = self
            .shares
            .write()
            .await
            .insert(key.clone(), project_id.to_string());
        debug_assert_eq!(res, None);
        Some(key)
    }

    pub async fn accept_share(&self, share_id: &str, target: &model::user::Model) -> bool {
        let project_id = {
            let mut shares = self.shares.write().await;
            let Some(project_id) = shares.remove(share_id) else {
                info!("Attempted to accept non-existent share: {}", share_id);
                return false;
            };
            project_id
        };

        if self.project_exists_for_user(&project_id, target).await {
            return true;
        }

        if !self.project_exists(&project_id).await {
            // Project has been deleted since sharing
            return false;
        }

        model::project_memberships::ActiveModel {
            user_id: Set(target.id),
            project_id: Set(project_id.to_string()),
            ..Default::default()
        }
        .insert(&self.conn)
        .await
        .is_ok()
    }

    pub async fn zip(&self, project_id: String) -> DuplexStream {
        self.project(project_id).await.zip().await
    }

    pub async fn project(&self, project_id: String) -> websocket::WebsocketProject {
        // TODO: write a more idiomatic version with a less ridiculous amount of cloning
        {
            let projects = self.projects.read().await;
            if let Some(project) = projects.get(&project_id) {
                return project.clone();
            }
        }
        let new_proj =
            websocket::WebsocketProject::load(self.conn.clone(), project_id.to_string()).await;
        let mut projects = self.projects.write().await;
        projects.insert(project_id.to_string(), new_proj.clone());
        new_proj
    }

    pub async fn gc(&self) {
        self.projects
            .write()
            .await
            .retain(|_, v| v.can_be_disposed())
    }
}

#[derive(Debug, PartialEq)]
pub enum CreateAccountResult {
    Created {
        id: i32,
    },
    Exists,
    InvalidToken,
    ExpiredToken,
    /// Name too short, password too weak
    BadRequest,
}

#[cfg(test)]
mod tests {
    use sea_orm::Database;

    use crate::app::{App, CreateAccountResult};
    use crate::model;

    #[tokio::test]
    async fn can_create_accounts() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

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
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

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
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

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
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

        let token1 = app.add_registration_token(None).await;
        let token2 = app.add_registration_token(None).await;
        assert_ne!(token1, token2);
    }

    #[tokio::test]
    async fn cant_login_with_wrong_password() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

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
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

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
        let from = app.user_from_id(user_id).await.unwrap();

        let projects = app.list_projects(&from).await;
        assert!(projects.is_some_and(|projects| projects.is_empty()));

        let res = app
            .insert_project("test_project".into(), user_id, "Test Title".into())
            .await;
        assert!(res, "Project creation failed");

        let projects = app.list_projects(&from).await.unwrap();
        assert_eq!(projects.len(), 1);

        let res = app
            .insert_project("test_project2".into(), user_id, "Test Title".into())
            .await;
        assert!(res, "Project creation failed");

        let projects = app.list_projects(&from).await.unwrap();
        assert_eq!(projects.len(), 2);

        let res = app
            .insert_project("test_project".into(), user_id, "Test Title".into())
            .await;
        assert!(!res, "Created duplicate projects");

        let projects = app.list_projects(&from).await.unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[tokio::test]
    async fn admins_can_issue_tokens() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;
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
        assert_eq!(user.invited_by.unwrap(), admin_id);
        assert_eq!(app.user_count().await, 2);
    }

    #[tokio::test]
    async fn users_can_create_projects() {
        let (app, _, user) = sample_app().await;

        assert_eq!(app.project_count().await, 0);
        assert_eq!(app.list_projects(&user).await.unwrap().len(), 0);

        let project = app
            .insert_project("cool_project".into(), user.id, "Cool Project".into())
            .await;
        assert!(project);

        assert_eq!(app.project_count().await, 1);
        assert_eq!(app.list_projects(&user).await.unwrap().len(), 1);
        let projects = app.list_projects(&user).await.unwrap();
        let project = projects.first().unwrap();
        assert_eq!(project.id, "cool_project");
        assert_eq!(project.title, "Cool Project");
        assert_eq!(project.owner_id, user.id);

        assert!(app.project_exists("cool_project").await);
        assert!(app.project_exists_for_user("cool_project", &user).await);
        assert_eq!(
            app.get_project_for_user("cool_project", &user)
                .await
                .unwrap(),
            project.clone()
        );
    }

    #[tokio::test]
    async fn users_can_share_projects_with_others() {
        let (app, admin, user) = sample_app().await;

        assert_eq!(app.list_projects(&admin).await.unwrap().len(), 0);
        assert_eq!(app.list_projects(&user).await.unwrap().len(), 0);

        app.insert_project("cool_project".into(), user.id, "Cool Project".into())
            .await;

        assert_eq!(app.list_projects(&admin).await.unwrap().len(), 0);
        assert_eq!(app.list_projects(&user).await.unwrap().len(), 1);

        let share = app.share("cool_project", &user).await.unwrap();
        assert_eq!(app.list_projects(&admin).await.unwrap().len(), 0);

        app.accept_share(&share, &admin).await;
        assert_eq!(app.list_projects(&admin).await.unwrap().len(), 1);
        assert_eq!(app.list_projects(&user).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn cant_share_foreign_projects() {
        let (app, admin, user) = sample_app().await;
        app.insert_project("cool_project".into(), user.id, "Cool Project".into())
            .await;
        let share = app.share("cool_project", &admin).await;
        assert!(share.is_none(), "Can't share when not member");

        let share = app.share("cool_project", &user).await.unwrap();
        app.accept_share(&share, &admin).await;

        let share = app.share("cool_project", &admin).await;
        assert!(share.is_none(), "Can't share when not owner");
    }

    /// Sets up app with one admin and one dummy user.
    async fn sample_app() -> (App, model::user::Model, model::user::Model) {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

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
