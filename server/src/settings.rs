use std::sync::atomic::AtomicBool;

pub struct ServerSettings {
    db: sqlx::SqlitePool,
    /// If true no registration tokens are needed to register.
    open_sign_ups: AtomicBool,
    /// Allow non-admin users to create projects
    user_projects: AtomicBool,
    /// Allow sharing projects with external users without account
    open_sharing: AtomicBool, // TODO: implement
}

impl ServerSettings {
    pub async fn new(db: sqlx::SqlitePool) -> Self {
        ServerSettings {
            open_sign_ups: setting::read(&db, "open_sign_ups").await
                .unwrap_or(AtomicBool::new(false)),
            user_projects: setting::read(&db, "user_projects").await
                .unwrap_or(AtomicBool::new(true)),
            open_sharing: setting::read(&db, "open_sharing").await
                .unwrap_or(AtomicBool::new(false)),
            db,
        }
    }

    pub fn open_sign_ups(&self) -> bool {
        self.open_sign_ups.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn set_open_sign_ups(&self, value: bool) -> Result<(), String> {
        setting::write(&self.db, "open_sign_ups", value).await?;
        self.open_sign_ups.store(value, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    pub fn user_projects(&self) -> bool {
        self.user_projects.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn set_user_projects(&self, value: bool) -> Result<(), String> {
        setting::write(&self.db, "user_projects", value).await?;
        self.user_projects.store(value, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    pub fn open_sharing(&self) -> bool {
        self.open_sharing.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn set_open_sharing(&self, value: bool) -> Result<(), String> {
        setting::write(&self.db, "open_sharing", value).await?;
        self.open_sharing.store(value, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

mod setting {
    pub(crate) async fn read<T>(db: &sqlx::SqlitePool, key: &str) -> Option<T>
    where T: serde::de::DeserializeOwned {
        sqlx::query!("SELECT value FROM server_settings WHERE key = ?", key)
            .fetch_optional(db)
            .await
            .expect("SQL failed")
            .map(|value| serde_json::from_slice::<T>(&value.value).ok())?
    }

    pub(crate) async fn write<T>(db: &sqlx::SqlitePool, key: &str, value: T) -> Result<(), String>
    where T: serde::Serialize {
        let value = serde_json::to_vec(&value)
            .map_err(|e| e.to_string())?;
        if let Err(e) = sqlx::query!(r#"
            INSERT INTO server_settings (key, value)
            VALUES (?, ?)
            ON CONFLICT (key) DO
                UPDATE SET value=excluded.value
            "#,
            key,
            value,
        ).execute(db).await {
            return Err(format!("Updating asset failed: {}", e))
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use crate::{app, settings::{ServerSettings, setting}};

    #[tokio::test]
    async fn reads_written_values() {
        let db = app::App::test_db().await.unwrap();

        assert!(setting::read::<i32>(&db, "key1").await.is_none());

        setting::write::<i32>(&db, "key1", 2).await.unwrap();
        assert_matches!(setting::read::<i32>(&db, "key1").await, Some(2));
        assert_matches!(setting::read::<bool>(&db, "key1").await, None);

        setting::write::<i32>(&db, "key1", 3).await.unwrap();
        assert_matches!(setting::read::<i32>(&db, "key1").await, Some(3));
        setting::write::<String>(&db, "key1", "^^".into()).await.unwrap();
        assert_eq!(setting::read::<String>(&db, "key1").await.unwrap(), "^^".to_string());

        assert_matches!(setting::read::<i32>(&db, "key2").await, None);
        setting::write::<i32>(&db, "key2", 4).await.unwrap();
        assert_matches!(setting::read::<i32>(&db, "key2").await, Some(4));
    }


    #[tokio::test]
    async fn server_settings_work_as_expected() {
        let db = app::App::test_db().await.unwrap();
        let settings = ServerSettings::new(db.clone()).await;

        assert_eq!(settings.open_sign_ups(), false);
        assert_eq!(settings.user_projects(), true);
        assert_eq!(settings.open_sharing(), false);

        settings.set_open_sign_ups(!false).await.unwrap();
        settings.set_user_projects(!true).await.unwrap();
        settings.set_open_sharing(!false).await.unwrap();

        assert_eq!(settings.open_sign_ups(), !false);
        assert_eq!(settings.user_projects(), !true);
        assert_eq!(settings.open_sharing(), !false);

        settings.set_open_sharing(false).await.unwrap();
        assert_eq!(settings.open_sharing(), false);
        _ = settings;

        let settings2 = ServerSettings::new(db).await;
        assert_eq!(settings2.open_sign_ups(), !false);
        assert_eq!(settings2.user_projects(), !true);
        assert_eq!(settings2.open_sharing(), false);

    }
}
