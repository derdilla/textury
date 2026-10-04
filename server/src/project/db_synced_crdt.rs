use std::sync::Arc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{error, warn};
use y_octo::{Doc, JwstCodecResult, Value};
use crate::{api::types::{self, ProjectId}, app::{self, EncodableDateTime}, project::filesystem::Filesystem};

/// Source of truth for a project's document tree, and assets.
pub struct DbSyncedCRDT {
    db: SqlitePool,
    project_id: ProjectId,
    doc: Arc<std::sync::RwLock<Doc>>,

}

impl DbSyncedCRDT {
    pub fn new(db: SqlitePool, project_id: ProjectId, doc: Doc) -> Self {
        Self {
            db,
            project_id,
            doc: Arc::new(std::sync::RwLock::new(doc)),
        }
    }

    // --- Input ---
    pub async fn apply_update(&self, message: impl AsRef<[u8]>, user: types::UserId) {
        let success = self.doc.write().unwrap().apply_update_from_binary_v1(message)
                .inspect_err(|e| warn!("Unable to apply update: {e}"))
                .is_ok();
        if success {
            // TODO(perf): store individual updates in DB with periodic batching
            self.store_doc_to_db(user).await;
        }
    }

    pub async fn set_asset(&self, uuid: String, data: Vec<u8>, user: types::UserId) {
        if let Err(e) = sqlx::query!(r#"
            INSERT INTO assets (project_id, asset_id, data)
            VALUES (?, ?, ?)
            ON CONFLICT (project_id, asset_id) DO
                UPDATE SET data=excluded.data
            "#,
            self.project_id,
            uuid,
            data,
        ).execute(&self.db).await {
            error!("Updating asset failed: {}", e)
        }

        let now = EncodableDateTime::now();
        sqlx::query!(
            "UPDATE projects
            SET last_modified = ?, last_modified_by = ?
            WHERE id = ?",
            now,
            user,
            self.project_id)
            .execute(&self.db).await
            .unwrap();
    }

    // --- Output ---

    pub fn id(&self) -> ProjectId {
        self.project_id
    }

    pub fn encode_update_v1(&self) -> JwstCodecResult<Vec<u8>> {
        let doc = self.doc.read().unwrap();
        doc.encode_update_v1()
    }


    /// Returns the path for a UUID
    pub fn lookup_uuid(&self, uuid: &str) -> Option<String> {
        let doc = self.doc.read().unwrap();
        let Ok(files) = doc.get_or_create_array("files") else {
            warn!("Missing property files in document");
            return None;
        };
        files.iter().find_map(|fs_entry| {
            let Value::Map(fs_entry) = fs_entry else {
                return None;
            };
            match YFileSnapshot::from_map(fs_entry) {
                Ok(v) => {
                    if v.uuid == uuid {
                        Some(v.full_path)
                    } else {
                        None
                    }
                },
                Err(err) => {
                    warn!("Failed to parse fs entry. Skipping: {err}");
                    None
                }
            }
        })
    }

    pub async fn write_to_fs(&self, fs: &impl Filesystem) -> Result<(), String> {
        for file in self.get_snapshots() {
            match file.entry_type {
                FileType::Directory => {
                    fs.create_dir(file.full_path.into()).await?;
                }
                FileType::File => {
                    fs.create_file(file.full_path.into(), &file.text.unwrap().into_bytes()).await?;
                }
                FileType::Asset => {
                    let query = sqlx::query!(
                        "SELECT data FROM assets WHERE project_id = ? AND asset_id = ?",
                        self.project_id, file.uuid
                    );
                    let Ok(Some(asset)) = query.fetch_optional(&self.db).await else {
                        // This can happen if the asset is still uploading while it is already
                        // added to the CRDT. I would consider this a client bug.
                        warn!("Unable to find asset {} in {}", file.uuid, self.project_id);
                        continue;
                    };
                    fs.create_file(file.full_path.into(), &asset.data).await?;
                }
            }
        }
        Ok(())
    }

    fn get_snapshots(&self) -> Vec<YFileSnapshot> {
        let doc = self.doc.read().unwrap();
        let files = doc.get_or_create_array("files")
            .expect("Unable to create files property in document");

        files.iter().filter_map(|fs_entry| {
            let Value::Map(fs_entry) = fs_entry else {
                return None;
            };
            match YFileSnapshot::from_map(fs_entry) {
                Ok(v) => Some(v),
                Err(err) => {
                    warn!("Failed to parse fs entry. Skipping: {err}");
                    None
                }
            }
        }).collect()
    }


    // --- Persistence ---

    async fn store_doc_to_db(&self, user: types::UserId) {
        let state = self.doc.read().unwrap().encode_update_v1().unwrap();
        let now = app::EncodableDateTime::now();
        sqlx::query!(
            "UPDATE projects
            SET last_state = ?, last_modified = ?, last_modified_by = ?
            WHERE id = ?",
            state,
            now,
            user,
            self.project_id)
            .execute(&self.db).await
            .unwrap();
    }
}

#[derive(Clone)]
struct YFileSnapshot {
    uuid: String,
    entry_type: FileType,
    full_path: String,
    text: Option<String>,
}

impl YFileSnapshot {
    pub fn from_map(fs_entry: y_octo::Map) -> Result<Self, &'static str> {
        // Decode the data. Note that only full_path and text need to be collaborative.
        let Some(Value::Any(y_octo::Any::String(version))) = fs_entry.get("version") else {
            return Err("Missing property: 'version'");
        };
        if version != "1" {
            return Err("Unrecognized file entry version");
        }

        let Some(Value::Any(y_octo::Any::String(uuid))) = fs_entry.get("uuid") else {
            return Err("Missing property: 'uuid'");
        };
        let Some(Value::Any(y_octo::Any::String(entry_type))) = fs_entry.get("type") else {
            return Err("Missing property: 'type'");
        };
        let Some(entry_type) = enum_from_str::<FileType>(&mut entry_type.to_string()) else {
            return Err("Can't decode property: 'type'");
        };
        let Some(Value::Text(full_path)) = fs_entry.get("full_path") else {
            return Err("Missing property: 'full_path'");
        };

        let text = if entry_type == FileType::File {
            let Some(Value::Text(text)) = fs_entry.get("text") else {
                return Err("Missing property for file type: 'text'");
            };
            Some(text.to_string())
        } else {
            None
        };

        Ok(YFileSnapshot {
            uuid,
            entry_type,
            full_path: full_path.to_string(),
            text,
        })
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
enum FileType {
    File,
    Directory,
    Asset,
}

fn enum_from_str<'a, T: Deserialize<'a>>(value: &'a mut String) -> Option<T> {
    // The normal serde_json implementation rejects any enum value not in quotes
    if !value.starts_with('"') {
        value.insert(0, '"');
    }
    if !value.ends_with('"') {
        value.insert(value.len(), '"');
    }
    serde_json::from_str(value).ok()
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[tokio::test]
    async fn updates_last_modified_on_set_asset() {
        let (app, admin, user) = crate::app::tests::sample_app().await;

        let pid = app.insert_project(user.id, "Foo".into()).await.unwrap();
        let share_id = app.share(pid, user.id).await.unwrap();
        assert!(app.accept_share(&share_id, admin.id).await);

        assert_eq!(app.list_projects(admin.id).await.unwrap().first().unwrap().last_modified_by, user.id);
        let ws_project = app.project(pid).await;
        let project = ws_project.get_project();

        assert_eq!(app.list_projects(admin.id).await.unwrap().first().unwrap().last_modified_by, user.id);
        project.crdt.set_asset("uuid".into(), Vec::new(), admin.id).await;
        assert_eq!(app.list_projects(admin.id).await.unwrap().first().unwrap().last_modified_by, admin.id);
        project.crdt.store_doc_to_db(user.id).await;
        assert_eq!(app.list_projects(admin.id).await.unwrap().first().unwrap().last_modified_by, user.id);
    }

    #[tokio::test]
    async fn writes_single_file_to_fs() {
        let db = app::App::test_db().await.unwrap();
        let doc = Doc::new();

        let mut arr = doc.get_or_create_array("files").unwrap();

        let mut map = doc.create_map().unwrap();
        map.insert("version".into(), "1").unwrap();
        map.insert("uuid".into(), "d0e2f77c-00b8-446c-85fc-0d638d2c5a48")
            .unwrap();
        map.insert(
            "type".into(),
            serde_json::to_string(&super::FileType::File).unwrap().as_str(),
        )
            .unwrap();
        let mut text = doc.create_text().unwrap();
        text.insert(0, "/file.txt").unwrap();
        map.insert("full_path".into(), text).unwrap();
        let mut text = doc.create_text().unwrap();
        text.insert(0, "Test content 1").unwrap();
        map.insert("text".into(), text).unwrap();

        arr.push(map).unwrap();

        let pid = ProjectId::random();
        let crdt = DbSyncedCRDT::new(db.clone(), pid, doc);
        let mut fs = DebugFS(Mutex::new(Vec::new()));

        assert!(fs.0.lock().unwrap().is_empty());
        crdt.write_to_fs(&mut fs).await.unwrap();
        assert_eq!(fs.0.lock().unwrap().first().unwrap().path, "/file.txt");
    }

    struct DebugFS(pub Mutex<Vec<FileMetadata>>);

    impl Filesystem for DebugFS {
        async fn create_file(&self, path: String, content: &Vec<u8>) -> Result<(), String> {
            self.0.lock().unwrap().push(FileMetadata { path, file_type: FileType::Text { size: content.len() } });
            Ok(())
        }

        async fn create_dir(&self, path: String) -> Result<(), String> {
            self.0.lock().unwrap().push(FileMetadata { path, file_type: FileType::Directory });
            Ok(())
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct FileMetadata {
        pub path: String,
        pub file_type: FileType,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum FileType {
        Text { size: usize },
        Directory,
    }
}
