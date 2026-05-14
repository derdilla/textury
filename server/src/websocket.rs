use crate::compilation::{CompilationResult, Compiler, compile};
use crate::{fs, model};
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, SelectExt, Set};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::time::Duration;
use tokio::io::DuplexStream;
use tokio::sync::{Mutex, RwLock};
use tracing::{error, warn};
use y_octo::{Doc, JwstCodecResult, Value};

const PING_TIMEOUT: Duration = Duration::from_millis(30000);

type WSStore = RwLock<HashMap<usize, Arc<Mutex<SplitSink<WebSocket, Message>>>>>;

#[derive(Clone)]
pub struct WebsocketProject {
    clients: Arc<WSStore>,
    counter: Arc<AtomicUsize>,
    inner: Arc<InnerProject>,
}

impl WebsocketProject {
    pub async fn load(db: DatabaseConnection, project_id: String) -> Self {
        let mut doc = Doc::new();

        // TODO: cleanup
        let project = model::project::Entity::find_by_id(&project_id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        if let Some(last) = project.last_state {
            doc.apply_update_from_binary_v1(last).unwrap();
        }
        let inner = InnerProject {
            id: project_id,
            db,
            doc: Arc::new(std::sync::RwLock::new(doc)),
            connections: Default::default(),
        };
        WebsocketProject {
            clients: Arc::new(Default::default()),
            counter: Arc::new(Default::default()),
            inner: Arc::new(inner),
        }
    }

    pub fn can_be_disposed(&self) -> bool {
        self.inner.connections.load(Ordering::SeqCst) == 0
    }

    pub async fn connect(&self, ws: WebSocket) {
        let (mut sink, stream) = ws.split();
        // Setup
        if let Ok(initial_update) = self.inner.encode_update_v1() {
            let encoded_update = BASE64_STANDARD.encode(&initial_update);
            let message = serde_json::to_string(&ServerMessage::Update {
                update: encoded_update,
            })
            .unwrap();
            _ = sink
                .send(Message::from(message))
                .await
                .inspect_err(|e| error!("Unable to send message to client: {e}"));
        } else {
            warn!("Failed to send initial update to connecting client");
        }

        let sink = Arc::new(Mutex::new(sink));
        let pong_received = Arc::new(AtomicBool::new(true));
        let connection_id = self.counter.fetch_add(1, Ordering::SeqCst);
        self.clients
            .write()
            .await
            .insert(connection_id, Arc::clone(&sink));

        // Ensure regular pings
        let pong_received_clone = pong_received.clone();
        let clients = self.clients.clone();
        let interval = tokio::task::spawn(async move {
            let mut interval = tokio::time::interval(PING_TIMEOUT);
            loop {
                interval.tick().await;
                if !pong_received_clone.load(Ordering::Relaxed) {
                    if let Some(removed) = clients.write().await.remove(&connection_id) {
                        _ = removed.lock().await.send(Message::Close(None)).await;
                    }
                    break;
                }
                // TODO: pong_received_clone.store(false, Ordering::Relaxed);

                if let Some(client) = clients.write().await.get(&connection_id) {
                    _ = client.lock().await.send(Message::Ping(Bytes::new())).await;
                }
            }
        });

        // Message handling
        let clients = self.clients.clone();
        stream
            .for_each_concurrent(40, async |message| {
                async fn others(
                    clients: &Arc<WSStore>,
                    connection_id: &usize,
                ) -> Vec<Arc<Mutex<SplitSink<WebSocket, Message>>>> {
                    let clients = clients.read().await;
                    if clients.len() <= 1 {
                        return Vec::new();
                    }
                    let mut other_list = Vec::with_capacity(clients.len() - 1);
                    for (key, other) in clients.iter() {
                        if *key != *connection_id {
                            other_list.push(other.clone());
                        }
                    }
                    other_list
                }

                async fn send_to(clients: &Arc<WSStore>, connection_id: &usize, message: Message) {
                    if let Some(client) = clients.read().await.get(connection_id) {
                        if client.lock().await.send(message).await.is_err() {
                            error!("Sending private message failed");
                        }
                    } else {
                        error!("Tried sending message to unknown connection: '{connection_id}'");
                    }
                }

                if let Ok(message) = message {
                    match message {
                        Message::Text(raw_message) => {
                            pong_received.fetch_or(true, Ordering::Relaxed);
                            let Ok(msg) = serde_json::from_str::<ClientMessage>(&raw_message)
                            else {
                                warn!("Received unexpected message: {:?}", raw_message);
                                return;
                            };
                            // Ignoring topic, signaling server is project specific
                            match msg {
                                ClientMessage::Update {
                                    update: encoded_update,
                                } => {
                                    let Ok(update) = BASE64_STANDARD.decode(&encoded_update) else {
                                        warn!("Received invalid base64 data");
                                        return;
                                    };
                                    self.inner.apply_update(&update).await;
                                    let message = serde_json::to_string(&ServerMessage::Update {
                                        update: encoded_update,
                                    })
                                    .unwrap();
                                    for client in others(&self.clients, &connection_id).await {
                                        _ = client
                                            .lock()
                                            .await
                                            .send(Message::from(message.clone()))
                                            .await
                                            .inspect_err(|e| {
                                                warn!("Error sending data update: {}", e)
                                            });
                                    }
                                }
                                ClientMessage::Awareness { update } => {
                                    let message =
                                        serde_json::to_string(&ServerMessage::Awareness { update })
                                            .unwrap();
                                    for client in others(&self.clients, &connection_id).await {
                                        _ = client
                                            .lock()
                                            .await
                                            .send(Message::from(message.clone()))
                                            .await
                                            .inspect_err(|e| {
                                                warn!("Error sending awareness update: {}", e)
                                            });
                                    }
                                }
                                ClientMessage::SetAsset { uuid, data } => {
                                    let Ok(data) = BASE64_STANDARD.decode(data) else {
                                        warn!("Received invalid base64 data");
                                        return;
                                    };
                                    self.inner.set_asset(uuid.clone(), data).await;
                                    let message =
                                        serde_json::to_string(&ServerMessage::AssetUpdated {
                                            uuid,
                                        })
                                        .unwrap();
                                    for client in others(&self.clients, &connection_id).await {
                                        _ = client
                                            .lock()
                                            .await
                                            .send(Message::from(message.clone()))
                                            .await
                                            .inspect_err(|e| {
                                                warn!("Error sending asset update: {}", e)
                                            });
                                    }
                                }
                                ClientMessage::Compile { uuid, compiler } => {
                                    match self.inner.compile(uuid.clone(), compiler).await {
                                        Err(e) => {
                                            // send log privately
                                            let log =
                                                serde_json::to_string(&ServerMessage::SetLog {
                                                    uuid,
                                                    data: e,
                                                })
                                                .unwrap();
                                            send_to(&clients, &connection_id, Message::from(log))
                                                .await
                                        }
                                        Ok(result) => {
                                            // Broadcast result
                                            let log =
                                                serde_json::to_string(&ServerMessage::SetLog {
                                                    uuid: uuid.clone(),
                                                    data: result.log,
                                                })
                                                .unwrap();
                                            let log = Message::from(log);
                                            for client in self.clients.read().await.values() {
                                                _ = client
                                                    .lock()
                                                    .await
                                                    .send(log.clone())
                                                    .await
                                                    .inspect_err(|e| {
                                                        warn!("Error sending log update: {}", e)
                                                    });
                                            }

                                            if let Some(doc) = result.document {
                                                let pdf = format!(
                                                    "data:application/pdf;base64,{}",
                                                    BASE64_STANDARD.encode(doc)
                                                );
                                                let message =
                                                    serde_json::to_string(&ServerMessage::SetPdf {
                                                        uuid,
                                                        content: Some(pdf),
                                                    })
                                                    .unwrap();
                                                for client in self.clients.read().await.values() {
                                                    _ = client
                                                        .lock()
                                                        .await
                                                        .send(Message::from(message.clone()))
                                                        .await
                                                        .inspect_err(|e| {
                                                            warn!("Error sending pdf update: {}", e)
                                                        });
                                                }
                                            } else {
                                                send_to(
                                                    &clients,
                                                    &connection_id,
                                                    Message::from(
                                                        serde_json::to_string(
                                                            &ServerMessage::SetPdf {
                                                                uuid,
                                                                content: None,
                                                            },
                                                        )
                                                        .unwrap(),
                                                    ),
                                                )
                                                .await
                                            }
                                            // only clear doc for compilation started, if compilation
                                        }
                                    };
                                }
                            }
                        }
                        Message::Binary(_) => {
                            warn!("No handling for binary signaling implemented");
                        }
                        Message::Ping(_) => {
                            if let Some(client) = clients.read().await.get(&connection_id) {
                                _ = client.lock().await.send(Message::Pong(Bytes::new())).await;
                            }
                        }
                        Message::Pong(_) => {
                            pong_received.fetch_or(true, Ordering::Relaxed);
                        }
                        Message::Close(_) => {
                            clients.write().await.remove(&connection_id);
                        }
                    };
                }
            })
            .await;

        interval.abort();
    }

    pub async fn zip(&self) -> DuplexStream {
        let dir = tempfile::tempdir().unwrap();
        let fs = fs::FS::new(dir.path().into());
        self.inner.write_to_fs(&fs, None).await;
        fs.zip()
    }
}

#[derive(Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(tag = "t")]
enum ClientMessage {
    #[serde(rename = "u")]
    Update {
        /// Base64 encoded yjs v1 update
        update: String,
    },
    #[serde(rename = "a")]
    Awareness {
        /// Base64 encoded y-awareness update
        update: String,
    },
    #[serde(rename = "setAsset")]
    SetAsset {
        uuid: String,
        /// Base64 encoded content of the asset
        data: String,
    },
    #[serde(rename = "compile")]
    Compile { uuid: String, compiler: Compiler },
}

#[derive(Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(tag = "t")]
enum ServerMessage {
    #[serde(rename = "u")]
    Update {
        /// Base64 encoded yjs v1 update
        update: String,
    },
    #[serde(rename = "a")]
    Awareness {
        /// Base64 encoded y-awareness update
        update: String,
    },
    #[serde(rename = "setLog")]
    SetLog { uuid: String, data: String },
    #[serde(rename = "setPdf")]
    SetPdf {
        uuid: String,
        content: Option<String>,
    },
    #[serde(rename = "assetUpdated")]
    AssetUpdated { uuid: String },
}

struct InnerProject {
    pub id: String,
    pub db: DatabaseConnection,
    pub doc: Arc<std::sync::RwLock<Doc>>,
    pub connections: AtomicU32,
}

impl InnerProject {
    pub async fn apply_update(&self, message: impl AsRef<[u8]>) {
        let success = {
            let mut doc = self.doc.write().unwrap();
            doc.apply_update_from_binary_v1(message)
                .inspect_err(|e| warn!("Unable to apply update: {e}"))
                .is_ok()
        };
        if success {
            // TODO: store individual updates in DB; batch more for perf reasons
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

    pub fn encode_update_v1(&self) -> JwstCodecResult<Vec<u8>> {
        let doc = self.doc.read().unwrap();
        doc.encode_update_v1()
    }

    fn get_snapshots(&self) -> Vec<YFileSnapshot> {
        let doc = self.doc.read().unwrap();

        // TODO: we could add and store a hash and skip all this work if we ever run into perf problems

        let Ok(files) = doc.get_or_create_array("files") else {
            warn!("Missing property files in document");
            return Vec::new();
        };

        let mut snapshots = Vec::new();
        for fs_entry in files.iter() {
            let Value::Map(fs_entry) = fs_entry else {
                continue;
            };
            match YFileSnapshot::from_map(fs_entry) {
                Ok(snapshot) => snapshots.push(snapshot),
                Err(e) => {
                    warn!("Failed to parse fs entry. Skipping: {e}");
                    continue;
                }
            }
        }
        snapshots
    }

    async fn write_to_fs(&self, fs: &fs::FS, snapshots: Option<Vec<YFileSnapshot>>) {
        let snapshots = snapshots.unwrap_or_else(|| self.get_snapshots());

        for file in snapshots {
            match file.entry_type {
                FileType::File => {
                    _ = fs
                        .create_file(file.full_path.into(), file.text.unwrap().into())
                        .await
                        .inspect_err(|e| error!("{e}"));
                }
                FileType::Directory => {
                    _ = fs
                        .create_dir(file.full_path.into())
                        .await
                        .inspect_err(|e| error!("{e}"));
                }
                FileType::Asset => {
                    let Ok(Some(asset)) = model::asset::Entity::find_by_id((
                        self.id.to_string(),
                        file.uuid.to_string(),
                    ))
                    .one(&self.db)
                    .await
                    else {
                        // TODO: we might have a race condition here
                        warn!("Unable to find asset {} in {}", file.uuid, self.id);
                        continue;
                    };
                    _ = fs
                        .create_file(file.full_path.into(), asset.data)
                        .await
                        .inspect_err(|e| error!("{e}"));
                }
            }
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

        self.update_last_modified().await;
    }

    /// Returns compilation result or private error on misuse.
    pub async fn compile(
        &self,
        uuid: String,
        compiler: Compiler,
    ) -> Result<CompilationResult, String> {
        let dir = tempfile::tempdir().unwrap();
        let fs = fs::FS::new(dir.path().into());
        let snapshots = self.get_snapshots();
        let Some(initial_path) = snapshots.iter().find(|e| e.uuid == uuid) else {
            return Err(format!("No file for {uuid}"));
        };
        self.write_to_fs(&fs, Some(snapshots.clone())).await;
        let document_path = match fs.validated_path(&initial_path.full_path) {
            Ok(v) => v,
            Err(e) => return Err(format!("Compile received for invalid path: {e}")),
        };

        let result = compile(fs.root(), &document_path, compiler).await;
        if result.document.is_some() {
            // update latest compilation
            let project = model::project::Entity::find_by_id(&self.id)
                .one(&self.db)
                .await;
            if let Ok(Some(project)) = project {
                let mut project: model::project::ActiveModel = project.into();
                project.latest_compilation = Set(result.document.clone());
                if let Err(e) = project.update(&self.db).await {
                    warn!("Failed updating last saved: {:?}", e);
                }
            }
        }

        Ok(result)
    }
    async fn update_last_modified(&self) {
        let now = time::OffsetDateTime::now_utc();
        let project = model::project::Entity::find_by_id(&self.id)
            .one(&self.db)
            .await;
        if let Ok(Some(project)) = project {
            let mut project: model::project::ActiveModel = project.into();
            project.last_modified = Set(time::PrimitiveDateTime::new(now.date(), now.time()));
            if let Err(e) = project.update(&self.db).await {
                warn!("Failed updating last saved: {:?}", e);
            }
        }
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

        let project = InnerProject {
            id: "foo_bar".to_string(),
            db,
            doc: Arc::new(std::sync::RwLock::new(doc)),
            connections: Default::default(),
        };

        let dir = tempfile::tempdir().unwrap();
        let fs = fs::FS::new(PathBuf::from(dir.path()));

        let fs::FileType::Directory { children } = fs.index().await.unwrap().file_type else {
            panic!()
        };
        assert!(children.is_empty());

        project.write_to_fs(&fs, None).await;

        let fs::FileType::Directory { children } = fs.index().await.unwrap().file_type else {
            panic!()
        };

        assert_eq!(children.first().unwrap().name, "file.txt");
    }
}
