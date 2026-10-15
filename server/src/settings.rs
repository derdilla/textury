use std::sync::atomic::{AtomicBool, AtomicU64};

pub struct ServerSettings {
    db: sqlx::SqlitePool,
    /// If true no registration tokens are needed to register.
    open_sign_ups: AtomicBool,
    /// Allow non-admin users to create projects
    user_projects: AtomicBool,
    /// Allow sharing projects with external users without account
    open_sharing: AtomicBool,
    /// Number of seconds a single compilation is allowed to take
    max_compile_duration: AtomicU64,
    /// Number of kilobytes a single compilation is allowed to take
    max_compile_memory: AtomicU64,
    /// Maximum number of projects any user is allowed to own before he can no longer create new projects
    max_project_count: AtomicU64,
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
            max_compile_duration: setting::read(&db, "max_compile_duration").await
                .unwrap_or(AtomicU64::new(60 * 10)),
            max_compile_memory: setting::read(&db, "max_compile_memory").await
                .unwrap_or(AtomicU64::new(100 * 1024)), // 100 MB
            max_project_count: setting::read(&db, "max_project_count").await
                .unwrap_or(AtomicU64::new(100)),
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

    pub fn max_compile_duration(&self) -> u64 {
        self.max_compile_duration.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn set_max_compile_duration(&self, value: u64) -> Result<(), String> {
        setting::write(&self.db, "max_compile_duration", value).await?;
        self.max_compile_duration.store(value, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    pub fn max_compile_memory(&self) -> u64 {
        self.max_compile_memory.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn set_max_compile_memory(&self, value: u64) -> Result<(), String> {
        setting::write(&self.db, "max_compile_memory", value).await?;
        self.max_compile_memory.store(value, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    pub fn max_project_count(&self) -> u64 {
        self.max_project_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn set_max_project_count(&self, value: u64) -> Result<(), String> {
        setting::write(&self.db, "max_project_count", value).await?;
        self.max_project_count.store(value, std::sync::atomic::Ordering::Relaxed);
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
        settings.set_max_compile_duration(4242).await.unwrap();
        settings.set_max_compile_memory(4243).await.unwrap();
        settings.set_max_project_count(123).await.unwrap();

        assert_eq!(settings.open_sign_ups(), !false);
        assert_eq!(settings.user_projects(), !true);
        assert_eq!(settings.open_sharing(), !false);
        assert_eq!(settings.max_compile_duration(), 4242);
        assert_eq!(settings.max_compile_memory(), 4243);
        assert_eq!(settings.max_project_count(), 123);

        settings.set_open_sharing(false).await.unwrap();
        assert_eq!(settings.open_sharing(), false);
        _ = settings;

        let settings2 = ServerSettings::new(db).await;
        assert_eq!(settings2.open_sign_ups(), !false);
        assert_eq!(settings2.user_projects(), !true);
        assert_eq!(settings2.open_sharing(), false);
        assert_eq!(settings2.max_compile_duration(), 4242);
        assert_eq!(settings2.max_compile_memory(), 4243);
        assert_eq!(settings2.max_project_count(), 123);

    }
}
