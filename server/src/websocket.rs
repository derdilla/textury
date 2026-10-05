use crate::api::types::{self, ProjectId, WsClientCompile, WsClientDocUpdate, WsClientMessage, WsClientSetAsset, WsServerAssetUpdated, WsServerAssetUpdatedT, WsServerAwareness, WsServerAwarenessT, WsServerDocUpdate, WsServerDocUpdateT, WsServerMessage, WsServerSetLog, WsServerSetPdf};
use crate::project::sandbox_manager::{SandboxManager};
use crate::project::{Project};
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use tokio::io::DuplexStream;
use tokio::sync::{Mutex, RwLock};
use tracing::{error, warn};
use y_octo::{Doc};
use crate::project::zip_fs::ZipFS;

const PING_TIMEOUT: Duration = Duration::from_millis(30000);

type WSStore = RwLock<HashMap<usize, Arc<Mutex<SplitSink<WebSocket, Message>>>>>;

#[derive(Clone)]
pub struct WebsocketProject<S: SandboxManager> {
    clients: Arc<WSStore>,
    counter: Arc<AtomicUsize>,
    inner: Arc<Project<S>>,
}

impl<S: SandboxManager> WebsocketProject<S> {
    pub async fn load(db: SqlitePool, project_id: ProjectId,
        sandbox_manager: Arc<S>,
    ) -> Self {
        let mut doc = Doc::new();

        // TODO: cleanup
        let project = sqlx::query!("SELECT last_state FROM projects WHERE id = ?", project_id)
            .fetch_one(&db).await.unwrap();
        if let Some(last) = project.last_state {
            doc.apply_update_from_binary_v1(last).unwrap();
        }
        let inner = Project::new(db, project_id, doc, sandbox_manager).await;

        WebsocketProject {
            clients: Arc::new(Default::default()),
            counter: Arc::new(Default::default()),
            inner: Arc::new(inner),
        }
    }

    #[cfg(test)]
    pub fn get_project(&self) -> Arc<Project<S>> {
        self.inner.clone()
    }

    pub fn can_be_disposed(&self) -> bool {
        self.inner.connections.load(Ordering::SeqCst) == 0
    }

    pub async fn connect(&self, ws: WebSocket, user: types::UserId) {
        let (mut sink, stream) = ws.split();
        // Setup
        if let Ok(initial_update) = self.inner.crdt.encode_update_v1() {
            let encoded_update = BASE64_STANDARD.encode(&initial_update);
            let message = serde_json::to_string(&WsServerMessage::DocUpdate(WsServerDocUpdate {
                t: WsServerDocUpdateT::U,
                update: encoded_update,
            }))
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
                            let Ok(msg) = serde_json::from_str::<WsClientMessage>(&raw_message)
                            else {
                                warn!("Received unexpected message: {:?}", raw_message);
                                return;
                            };
                            // Ignoring topic, signaling server is project specific
                            match msg {
                                WsClientMessage::DocUpdate (WsClientDocUpdate {
                                    update: encoded_update, ..
                                }) => {
                                    let Ok(update) = BASE64_STANDARD.decode(&encoded_update) else {
                                        warn!("Received invalid base64 data");
                                        return;
                                    };
                                    self.inner.crdt.apply_update(&update, user).await;
                                    let message = serde_json::to_string(&WsServerMessage::DocUpdate(WsServerDocUpdate {
                                        t: WsServerDocUpdateT::U,
                                        update: encoded_update
                                    }))
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
                                WsClientMessage::Awareness(update) => {
                                    let message =
                                        serde_json::to_string(&WsServerMessage::Awareness(WsServerAwareness {
                                            t: WsServerAwarenessT::A, update: update.update
                                        })).unwrap();
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
                                WsClientMessage::SetAsset(WsClientSetAsset { uuid, data, .. })  => {
                                    let Ok(data) = BASE64_STANDARD.decode(data) else {
                                        warn!("Received invalid base64 data");
                                        return;
                                    };
                                    self.inner.crdt.set_asset(uuid.clone(), data, user).await;
                                    let message =
                                        serde_json::to_string(&WsServerMessage::AssetUpdated(WsServerAssetUpdated {
                                            t: WsServerAssetUpdatedT::AssetUpdated, uuid
                                        })).unwrap();
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
                                WsClientMessage::Compile(WsClientCompile{ uuid, compiler, .. }) => {
                                    match self.inner.compile(uuid.clone(), compiler.into()).await {
                                        Err(e) => {
                                            // send log privately
                                            let log =
                                                serde_json::to_string(&WsServerMessage::SetLog(WsServerSetLog {
                                                    t: types::WsServerSetLogT::SetLog,
                                                    uuid,
                                                    data: e,
                                                }))
                                                .unwrap();
                                            send_to(&clients, &connection_id, Message::from(log))
                                                .await
                                        }
                                        Ok(result) => {
                                            // Broadcast result
                                            let log =
                                                serde_json::to_string(&WsServerMessage::SetLog(WsServerSetLog {
                                                    t: types::WsServerSetLogT::SetLog,
                                                    uuid: uuid.clone(),
                                                    data: result.log,
                                                }))
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
                                                    serde_json::to_string(&WsServerMessage::SetPdf(WsServerSetPdf {
                                                        t: types::WsServerSetPdfT::SetPdf,
                                                        uuid,
                                                        content: Some(pdf),
                                                    }))
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
                                                            &WsServerMessage::SetPdf(WsServerSetPdf {
                                                                t: types::WsServerSetPdfT::SetPdf,
                                                                uuid,
                                                                content: None,
                                                            }),
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

    pub async fn zip(&self, writer: DuplexStream) {
        let mut fs = ZipFS::new(writer);
        if let Err(e) = self.inner.crdt.write_to_fs(&mut fs).await {
            error!("Zipping file failed: {e}")
        }
    }
}
