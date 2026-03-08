use rand::distr::SampleString;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, ModelTrait, SelectExt,
};
use time::OffsetDateTime;
use time::ext::NumericalDuration;

use crate::model;

#[derive(Clone)]
pub struct App {
    conn: DatabaseConnection,
}

impl App {
    pub async fn new(conn: DatabaseConnection) -> Self {
        let obj = Self { conn };
        obj.setup_schema().await;
        obj
    }

    async fn setup_schema(&self) {
        self.conn
            .get_schema_builder()
            .register(model::user::Entity)
            .register(model::project::Entity)
            .register(model::project_memberships::Entity)
            .register(model::registration_tokens::Entity)
            .apply(&self.conn)
            .await
            .unwrap()
    }

    pub async fn add_registration_token(&self) -> String {
        let token = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 20);
        let expires = OffsetDateTime::now_utc();
        let expires = expires.checked_add(15.minutes()).unwrap();
        model::registration_tokens::ActiveModel {
            token: Set(token.to_string()),
            expires: Set(expires),
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
        if token.expires < OffsetDateTime::now_utc() {
            token.delete(&self.conn).await.unwrap();
            return CreateAccountResult::ExpiredToken;
        }
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
        println!("User ID: {:?}", user);

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

    pub async fn insert_project(&self, project_id: String, user_id: i32) -> bool {
        use sea_orm::Set;

        let new_project = model::project::ActiveModel {
            id: Set(project_id.clone()),
            owner_id: Set(user_id),
        };
        if !new_project.insert(&self.conn).await.is_ok() {
            return false;
        }

        model::project_memberships::ActiveModel {
            user_id: Set(user_id),
            project_id: Set(project_id),
        }
        .insert(&self.conn)
        .await
        .is_ok()
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

    #[tokio::test]
    async fn can_create_accounts() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

        let token = app.add_registration_token().await;
        let res = app
            .register(
                token,
                "foobar".into(),
                "foo@bar.de".into(),
                "foobar!1sadf".into(),
            )
            .await;
        assert!(
            match res {
                CreateAccountResult::Created { .. } => true,
                _ => false,
            },
            "{:?}",
            res,
        );
    }

    #[tokio::test]
    async fn can_login() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

        let token = app.add_registration_token().await;
        let res = app
            .register(
                token,
                "foobar".into(),
                "foo@bar.de".into(),
                "foobar!1sadf".into(),
            )
            .await;
        assert!(
            match res {
                CreateAccountResult::Created { .. } => true,
                _ => false,
            },
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

        let token1 = app.add_registration_token().await;
        let token2 = app.add_registration_token().await;
        assert!(token1 != token2);
    }

    #[tokio::test]
    async fn cant_login_with_wrong_password() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

        let token = app.add_registration_token().await;
        let res = app
            .register(
                token,
                "foobar".into(),
                "foo@bar.de".into(),
                "foobar!1sadf".into(),
            )
            .await;
        assert!(
            match res {
                CreateAccountResult::Created { .. } => true,
                _ => false,
            },
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

        let token = app.add_registration_token().await;
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

        let res = app.insert_project("test_project".into(), user_id).await;
        assert!(res, "Project creation failed");

        let projects = app.list_projects(&from).await.unwrap();
        assert_eq!(projects.len(), 1);

        let res = app.insert_project("test_project2".into(), user_id).await;
        assert!(res, "Project creation failed");

        let projects = app.list_projects(&from).await.unwrap();
        dbg!(&projects);
        assert_eq!(projects.len(), 2);

        let res = app.insert_project("test_project".into(), user_id).await;
        assert!(!res, "Created duplicate projects");

        let projects = app.list_projects(&from).await.unwrap();
        assert_eq!(projects.len(), 2);
    }
}
