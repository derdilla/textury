mod login;
mod project;

use crate::api::login::{KEY, USER_COOKIE_NAME};
use crate::model;
use crate::model::User;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::Router;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, SelectExt, Set};
use time::ext::NumericalDuration;
use time::OffsetDateTime;
use tower_cookies::Cookies;

pub async fn router() -> Router<App> {
    let router = Router::new();
    // TODO: unified path prefixes?
    let router = login::router(router);
    let router = project::router(router).await;
    router
}


#[derive(Clone)]
pub struct App {
    conn: DatabaseConnection,
}

impl App {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self {
            conn,
        }
    }

    pub async fn add_registration_token(&self) -> String {
        let token = "TODO-secure-token-generation";
        let expires = OffsetDateTime::now_utc();
        let expires = expires.checked_add(15.minutes()).unwrap();
        model::registration_tokens::ActiveModel {
            token: Set(token.to_string()),
            expires: Set(expires),
        }.save(&self.conn).await.unwrap();

        token.to_string()
    }

    pub async fn register(&self, registration_token: String, name: String, email: String, password: String) -> CreateAccountResult {
        if name.len() < 4 || password.len() < 8 {
            return CreateAccountResult::BadRequest;
        }

        // Check for conflicts
        if model::user::Entity::find_by_email(&email).exists(&self.conn).await.unwrap() {
            return CreateAccountResult::Exists;
        }

        // Validate registration token
        let Ok(Some(token)) = model::registration_tokens::Entity::find_by_token(&registration_token).one(&self.conn).await else {
            return CreateAccountResult::InvalidToken
        };
        if token.expires < OffsetDateTime::now_utc() {
            token.delete(&self.conn).await.unwrap();
            return CreateAccountResult::ExpiredToken;
        }
        token.delete(&self.conn).await.unwrap();

        // Save account
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let Ok(password_hash) = argon2.hash_password(password.as_bytes(), &salt) else {
            return CreateAccountResult::BadRequest;
        };
        let new_user = model::user::ActiveModel {
            name: Set(name),
            email: Set(email),
            password_salt: Set(salt.to_string()), // TODO: we probably don't need to save this
            password_hash: Set(password_hash.to_string()),
            ..Default::default()
        };
        let new_user = new_user.save(&self.conn).await.unwrap();

        CreateAccountResult::Created { id: new_user.id.unwrap() }
    }

    /// Returns the user or none if the user doesn't exist or the password is wrong
    pub async fn login(&self, email: String, password: String) -> Option<model::user::Model> {
        let user_details = model::user::Entity::find_by_email(&email)
            .one(&self.conn)
            .await.unwrap()?;
        let parsed_hash = PasswordHash::new(&user_details.password_hash).ok()?;

        let argon2 = Argon2::default();
        argon2.verify_password(password.as_bytes(), &parsed_hash).ok()?;

        Some(user_details)
    }

    /// Gets user if in DB and id is in cookie.
    pub async fn user(&self, cookies: Cookies) -> Option<model::user::Model> {
        let key = KEY.get().unwrap();
        let private_cookies = cookies.private(key);
        let Some(user_cookie) = private_cookies.get(USER_COOKIE_NAME) else {
            return None;
        };
        let user_id = user_cookie.value().parse::<i32>().unwrap();
        println!("User ID: {}", user_id);

        let user = User::find_by_id(user_id)
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
            .await.unwrap()
    }

    pub async fn insert_project(&self, project_id: String, user_id: i32) -> bool {
        use sea_orm::Set;
        debug_assert!(!self.project_exists(&project_id).await, "Should be called manually");

        let new_project = model::project::ActiveModel {
            id: Set(project_id),
            owner_id: Set(user_id),
        };

        model::Project::insert(new_project)
            .exec(&self.conn)
            .await
            .is_ok()

        // TODO: update list in user
    }

    pub async fn list_projects(&self, from: model::user::Model) -> Option<Vec<model::project::Model>> {
        from.find_related(model::project::Entity).all(&self.conn)
            .await
            .ok()
    }
}

pub enum CreateAccountResult {
    Created{ id: i32 },
    Exists,
    InvalidToken,
    ExpiredToken,
    /// Name too short, password too weak
    BadRequest,
}
