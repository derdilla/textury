use rand::distr::SampleString;
use sqlx::{SqlitePool};
use tokio::task::{AbortHandle};
use tracing::error;

use crate::{api::{self, types}, app};

pub const DEFAULT_REGISTRATION_EXPIRERY: time::SignedDuration = time::SignedDuration::hours(48);
pub const DEFAULT_PROJECT_JOIN_EXPIRERY: time::SignedDuration = time::SignedDuration::days(7);

pub struct TokenManager {
    db: SqlitePool,
    worker: AbortHandle,
}

impl TokenManager {
    pub fn new(db: SqlitePool) -> Self {
        let worker = {
            let db = db.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_hours(6));
                loop {
                    interval.tick().await;
                    let now = app::EncodableDateTime::now();
                    if let Err(e) = sqlx::query!("DELETE FROM registration_tokens WHERE expires < ?", now)
                        .execute(&db).await
                    {
                        error!("Token cleanup failed: {}", e);
                    }
                    if let Err(e) = sqlx::query!("DELETE FROM project_join_tokens WHERE expires < ?", now)
                        .execute(&db).await
                    {
                        error!("Token cleanup failed: {}", e);
                    }
                }
            }).abort_handle()
        };
        TokenManager {
            db,
            worker,
        }
    }

    pub async fn add_registration_token(&self, issuer_id: Option<types::UserId>) -> String {
        let token = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 20);
        let expires = time::OffsetDateTime::now_utc();
        let expires = app::EncodableDateTime(expires.checked_add(DEFAULT_REGISTRATION_EXPIRERY).unwrap());
        sqlx::query!("INSERT INTO registration_tokens (token, expires, issuer_id) VALUES (?, ?, ?)",
            token,
            expires,
            issuer_id,
        )
        .execute(&self.db)
        .await
        .unwrap();

        token
    }

    pub async fn add_project_join_token(&self, project: types::ProjectId, issuer: types::UserId) -> String {
        let token = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 20);
        let expires = time::OffsetDateTime::now_utc();
        let expires = app::EncodableDateTime(expires.checked_add(DEFAULT_PROJECT_JOIN_EXPIRERY).unwrap());
        sqlx::query!("INSERT INTO project_join_tokens (token, expires, project_id, issuer_id) VALUES (?, ?, ?, ?)",
            token,
            expires,
            project,
            issuer,
        )
        .execute(&self.db)
        .await
        .unwrap();

        token
    }

    /// Invalidates [token] by removing it from the database.
    pub async fn try_use_registration_token(&self, token: &str) -> TokenUseResult<Option<types::UserId>> {
        let Ok(Some(token)) = sqlx::query!("DELETE FROM registration_tokens WHERE token = ? RETURNING expires, issuer_id", token)
            .fetch_optional(&self.db)
            .await else {
                return TokenUseResult::NotFound;
            };

        let expires = app::EncodableDateTime::from(token.expires).0;
        if expires < time::OffsetDateTime::now_utc() {
            TokenUseResult::Expired
        } else if let Some(issuer_id) = token.issuer_id {
            TokenUseResult::Valid(Some(issuer_id))
        } else {
            TokenUseResult::Valid(None)
        }
    }

    /// Invalidates [token] by removing it from the database.
    pub async fn try_use_project_join_token(&self, token: &str) -> TokenUseResult<(types::UserId, types::ProjectId)> {
        let Ok(Some(token)) = sqlx::query!("DELETE FROM project_join_tokens WHERE token = ?
            RETURNING expires, project_id, issuer_id", token)
            .fetch_optional(&self.db)
            .await else {
                return TokenUseResult::NotFound;
            };

        let expires = app::EncodableDateTime::from(token.expires).0;
        if expires < time::OffsetDateTime::now_utc() {
            TokenUseResult::Expired
        } else {
            TokenUseResult::Valid((token.issuer_id, token.project_id))
        }
    }

    pub async fn preview_project_join_token(&self, token: &str) -> Option<api::types::JoinProjectPreview> {
        let result = sqlx::query!(r#"
            SELECT T.expires,
                   T.project_id,
                   P.title AS project_title,
                   P.owner_id,
                   U.name AS owner_name,
                   P.created_at,
                   P.last_modified
               FROM project_join_tokens T
               JOIN projects P ON T.project_id = P.id
               JOIN users U ON P.owner_id = U.id
               WHERE token = ?"#, token)
            .fetch_optional(&self.db).await.unwrap()?;
        let expires = app::EncodableDateTime::from(result.expires).0;
        if expires < time::OffsetDateTime::now_utc() {
            None
        } else {
            Some(api::types::JoinProjectPreview {
                id: result.project_id,
                title: result.project_title,
                created_at: app::EncodableDateTime::from(result.created_at).0,
                updated_at: app::EncodableDateTime::from(result.last_modified).0,
                owner: api::types::User {
                    id: result.owner_id,
                    username: result.owner_name,
                },
            })
        }
    }
}

impl Drop for TokenManager {
    fn drop(&mut self) {
        self.worker.abort();
    }
}

#[derive(Debug)]
pub enum TokenUseResult<T> {
    NotFound,
    Expired,
    Valid(T)
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use crate::{api::types::{ProjectId, UserId}, app, token_manager::{TokenManager, TokenUseResult}};

    async fn setup() -> (UserId, TokenManager, ProjectId) {
        let db = app::App::test_db().await.unwrap();
        let uid = sqlx::query!("INSERT INTO users (name, email, password_hash, is_admin, invited_by)
            VALUES ('a', 'b', 'c', 1, NULL) RETURNING id").fetch_one(&db).await.unwrap();
        let uid = uid.id;
        let now = app::EncodableDateTime::now();
        let pid = ProjectId::random();
        sqlx::query!("INSERT INTO projects (id, title, owner_id, last_modified, last_modified_by, created_at)
            VALUES (?, 'f', ?, ?, ?, ?)", pid, uid, now, uid, now)
            .execute(&db).await.unwrap();
        (uid, TokenManager::new(db), pid)
    }

    #[tokio::test]
    async fn test_registration_token_valid_with_issuer() {
        let (issuer, manager, _pid) = setup().await;

        let token = manager.add_registration_token(Some(issuer)).await;
        let result = manager.try_use_registration_token(&token).await;

        assert_matches!(result, TokenUseResult::Valid(Some(_)));
    }

    #[tokio::test]
    async fn test_registration_token_valid_without_issuer() {
        let (_, manager, _pid) = setup().await;

        let token = manager.add_registration_token(None).await;
        let result = manager.try_use_registration_token(&token).await;

        assert_matches!(result, TokenUseResult::Valid(None));
    }

    #[tokio::test]
    async fn test_invalid_token_returns_not_found() {
        let (_, manager, _pid) = setup().await;

        let reg_result = manager.try_use_registration_token("foobar").await;
        let proj_result = manager.try_use_project_join_token("foobar").await;

        assert_matches!(reg_result, TokenUseResult::NotFound);
        assert_matches!(proj_result, TokenUseResult::NotFound);
    }

    #[tokio::test]
    async fn test_project_join_token_valid() {
        let (issuer, manager, pid) = setup().await;

        let token = manager.add_project_join_token(pid, issuer).await;
        let result = manager.try_use_project_join_token(&token).await;

        let TokenUseResult::Valid((iss2, pid2)) = result else { panic!() };
        assert_eq!(issuer, iss2);
        assert_eq!(pid, pid2);
    }

    #[tokio::test]
    async fn test_cross_use_impossible() {
        let (issuer, manager, _) = setup().await;

        let token = manager.add_registration_token(Some(issuer)).await;
        let result = manager.try_use_project_join_token(&token).await;

        assert_matches!(result, TokenUseResult::NotFound);
    }
}
