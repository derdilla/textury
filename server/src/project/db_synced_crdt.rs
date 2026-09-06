use std::sync::Arc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, SelectExt, Set};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};
use y_octo::{Doc, JwstCodecResult, Value};
use crate::model;
use crate::project::filesystem::Filesystem;

/// Source of truth for a project's document tree, and assets.
pub struct DbSyncedCRDT {
    db: DatabaseConnection,
    /// Project ID
    id: String,
    doc: Arc<std::sync::RwLock<Doc>>,

}

impl DbSyncedCRDT {
    pub fn new(db: DatabaseConnection, id: String, doc: Doc) -> Self {
        Self {
            db,
            id,
            doc: Arc::new(std::sync::RwLock::new(doc)),
        }
    }

    // --- Input ---
    pub async fn apply_update(&self, message: impl AsRef<[u8]>) {
        let success = self.doc.write().unwrap().apply_update_from_binary_v1(message)
                .inspect_err(|e| warn!("Unable to apply update: {e}"))
                .is_ok();
        if success {
            // TODO(perf): store individual updates in DB with periodic batching
            self.store_doc_to_db().await;
        }
    }

    pub async fn set_asset(&self, uuid: String, data: Vec<u8>) {
        let key = (self.id.to_string(), uuid.to_string());
        let exists = model::asset::Entity::find_by_id(key)
            .exists(&self.db)
            .await
            .inspect_err(|e| error!("Lookup failed: {e:?}"))
            .unwrap_or(false);

        let model = model::asset::ActiveModel {
            project_id: Set(self.id.to_string()),
            asset_id: Set(uuid.to_string()),
            data: Set(data),
        };
        if exists {
            if let Err(e) = model.update(&self.db).await {
                error!("Failed updating asset: {e:?}");
            }
        } else {
            if let Err(e) = model.insert(&self.db).await {
                error!("Failed inserting asset: {e:?}");
            }
        }
    }

    // TODO: reimplement last_modified update: called from all the right places and add who did it

    // --- Output ---

    pub fn id(&self) -> &str {
        self.id.as_str()
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
                    let Ok(Some(asset)) = model::asset::Entity::find_by_id((
                        self.id.to_string(),
                        file.uuid.to_string(),
                    ))
                        .one(&self.db)
                        .await
                    else {
                        // This can happen if the asset is still uploading while it is already
                        // added to the CRDT. I would consider this a client bug.
                        warn!("Unable to find asset {} in {}", file.uuid, self.id);
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

    async fn store_doc_to_db(&self) {
        let state = self.doc.read().unwrap().encode_update_v1().unwrap();

        let mut project = model::project::Entity::find_by_id(&self.id)
            .one(&self.db)
            .await
            .unwrap()
            .unwrap()
            .into_active_model();
        project.last_state = Set(Some(state));
        project.update(&self.db).await.unwrap();
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
    use super::*;
    use sea_orm::DatabaseBackend;
    use sea_orm::MockDatabase;
    use std::path::PathBuf;
    use crate::project::disk_fs;

    #[tokio::test]
    async fn writes_single_file_to_fs() {
        let db = MockDatabase::new(DatabaseBackend::Sqlite).into_connection();
        let doc = Doc::new();

        let mut arr = doc.get_or_create_array("files").unwrap();

        let mut map = doc.create_map().unwrap();
        map.insert("version".into(), "1").unwrap();
        map.insert("uuid".into(), "d0e2f77c-00b8-446c-85fc-0d638d2c5a48")
            .unwrap();
        map.insert(
            "type".into(),
            serde_json::to_string(&FileType::File).unwrap().as_str(),
        )
            .unwrap();
        let mut text = doc.create_text().unwrap();
        text.insert(0, "/file.txt").unwrap();
        map.insert("full_path".into(), text).unwrap();
        let mut text = doc.create_text().unwrap();
        text.insert(0, "Test content 1").unwrap();
        map.insert("text".into(), text).unwrap();

        arr.push(map).unwrap();

        let crdt = DbSyncedCRDT::new(db.clone(), "foo_bar".to_string(), doc);

        // TODO: non-physical fs
        let dir = tempfile::tempdir().unwrap();
        let mut fs = disk_fs::DiskFS::new(PathBuf::from(dir.path()));

        let disk_fs::FileType::Directory { children } = fs.index().await.unwrap().file_type else {
            panic!()
        };
        assert!(children.is_empty());

        crdt.write_to_fs(&mut fs).await.unwrap();

        let disk_fs::FileType::Directory { children } = fs.index().await.unwrap().file_type else {
            panic!()
        };

        assert_eq!(children.first().unwrap().name, "file.txt");
    }
}
