use crate::api::types::{self, ProjectId, WsClientCompile, WsClientDocUpdate, WsClientMessage, WsClientSetAsset, WsServerAssetUpdated, WsServerAssetUpdatedT, WsServerAwareness, WsServerAwarenessT, WsServerDocUpdate, WsServerDocUpdateT, WsServerLog, WsServerMessage, WsServerResetLog, WsServerSetPdf};
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
use tokio::sync::{Mutex, RwLock, mpsc};
use tracing::{error, trace, warn};
use crate::project::zip_fs::ZipFS;

const PING_TIMEOUT: Duration = Duration::from_millis(30000);

type WSStore = RwLock<HashMap<usize, Arc<Connection>>>;

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
        WebsocketProject {
            clients: Default::default(),
            counter: Default::default(),
            inner: Arc::new(Project::new(db, project_id, sandbox_manager).await),
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
        let (sink, stream) = ws.split();
        let client = Arc::new(Connection::new(sink));
        // Setup
        if let Ok(initial_update) = self.inner.crdt.encode_update_v1() {
            let encoded_update = BASE64_STANDARD.encode(&initial_update);
            client.send(&WsServerMessage::DocUpdate(WsServerDocUpdate {
                t: WsServerDocUpdateT::U,
                update: encoded_update,
            })).await;
        } else {
            warn!("Failed to send initial update to connecting client");
        }

        let pong_received = Arc::new(AtomicBool::new(true));
        let connection_id = self.counter.fetch_add(1, Ordering::SeqCst);
        self.clients
            .write()
            .await
            .insert(connection_id, client.clone());

        // Ensure regular pings
        let pong_received_clone = pong_received.clone();
        let clients = self.clients.clone();
        let ping_client = client.clone();
        let interval = tokio::task::spawn(async move {
            let mut interval = tokio::time::interval(PING_TIMEOUT);
            loop {
                interval.tick().await;
                if !pong_received_clone.load(Ordering::Relaxed) {
                    if let Some(removed) = clients.write().await.remove(&connection_id) {
                        _ = removed.send_raw(Message::Close(None)).await;
                    }
                    break;
                }
                pong_received_clone.store(false, Ordering::Relaxed);
                ping_client.send_raw(Message::Ping(Bytes::new())).await;
            }
        });

        stream.for_each_concurrent(40, async |message| {
            let Ok(message) = message else { return; };
            match message {
                Message::Text(raw_message) => {
                    pong_received.fetch_or(true, Ordering::Relaxed);
                    let Ok(msg) = serde_json::from_str::<WsClientMessage>(&raw_message)
                    else {
                        warn!("Received unexpected message: {:?}", raw_message);
                        return;
                    };
                    self.handle_message(msg, connection_id, user, client.clone()).await;
                }
                Message::Binary(_) => {
                    warn!("No handling for binary signaling implemented");
                }
                Message::Ping(_) => {
                    _ = client.send_raw(Message::Pong(Bytes::new())).await;
                }
                Message::Pong(_) => {
                    pong_received.fetch_or(true, Ordering::Relaxed);
                }
                Message::Close(_) => {
                    self.clients.write().await.remove(&connection_id);
                }
            };
        }).await;

        interval.abort();
    }

    async fn handle_message(&self, msg: WsClientMessage, connection_id: usize, user: types::UserId, client: Arc<Connection>) {
        match msg {
            WsClientMessage::DocUpdate (WsClientDocUpdate {
                update: encoded_update, ..
            }) => {
                let Ok(update) = BASE64_STANDARD.decode(&encoded_update) else {
                    warn!("Received invalid base64 data");
                    return;
                };
                self.inner.crdt.apply_update(&update, user).await;
                send_to_others(&self.clients, &connection_id, &WsServerMessage::DocUpdate(WsServerDocUpdate {
                    t: WsServerDocUpdateT::U,
                    update: encoded_update
                })).await;
            }
            WsClientMessage::Awareness(update) => {
                send_to_others(&self.clients, &connection_id, &WsServerMessage::Awareness(WsServerAwareness {
                    t: WsServerAwarenessT::A, update: update.update
                })).await;
            }
            WsClientMessage::SetAsset(WsClientSetAsset { uuid, data, .. })  => {
                let Ok(data) = BASE64_STANDARD.decode(data) else {
                    warn!("Received invalid base64 data");
                    return;
                };
                self.inner.crdt.set_asset(uuid.clone(), data, user).await;
                send_to_others(&self.clients, &connection_id, &WsServerMessage::AssetUpdated(WsServerAssetUpdated {
                    t: WsServerAssetUpdatedT::AssetUpdated, uuid
                })).await;
            }
            WsClientMessage::Compile(WsClientCompile{ uuid, compiler, .. }) => {
                let (tx, mut rx) = mpsc::channel(8);
                let log_client = client.clone();
                let requester_uuid = uuid.clone();
                let compilation = self.inner.compile(uuid.clone(), compiler.into(), tx.clone());
                let log_handle = tokio::spawn(async move {
                    while let Some(msg) = rx.recv().await {
                        log_client.send(&WsServerMessage::Log(WsServerLog {
                            t: types::WsServerLogT::Log,
                            uuid: requester_uuid.clone(),
                            data: msg,
                        })).await;
                    }
                    trace!("logging channel closed");
                });
                client.send(&WsServerMessage::ResetLog(WsServerResetLog {
                    t: types::WsServerResetLogT::ResetLog,
                    uuid: uuid.clone(),
                })).await;
                match compilation.await {
                    Err(e) => {
                        _ = tx.send(format!("Execution error: {}", e)).await;
                    }
                    Ok(result) => {
                        // Broadcast result
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
                                client.send_raw(Message::from(message.clone())).await;
                            }
                        } else {
                            client.send(&WsServerMessage::SetPdf(WsServerSetPdf {
                                t: types::WsServerSetPdfT::SetPdf,
                                uuid,
                                content: None,
                            })).await;
                        }
                        // only clear doc for compilation started, if compilation
                    }
                };
                log_handle.abort();
            }
        }
    }

    pub async fn zip(&self, writer: DuplexStream) {
        let mut fs = ZipFS::new(writer);
        if let Err(e) = self.inner.crdt.write_to_fs(&mut fs).await {
            error!("Zipping file failed: {e}")
        }
    }
}

struct Connection(Mutex<SplitSink<WebSocket, Message>>);

impl Connection {
    pub fn new(sink: SplitSink<WebSocket, Message>) -> Self {
        Connection(Mutex::new(sink))
    }

    pub fn send(&self, message: &WsServerMessage) -> impl Future<Output = ()> {
        let serialized = serde_json::to_string(message).unwrap();
        let serialized = Message::from(serialized);
        self.send_raw(serialized)
    }

    pub async fn send_raw(&self, message: Message) {
        let mut sink = self.0.lock().await;
        if let Err(err) = sink.send(message).await {
            warn!("Unable to send message: {}", err);
        }
    }
}

async fn send_to_others(
    clients: &WSStore,
    connection_id: &usize,
    message: &WsServerMessage,
) {
    let clients = clients.read().await;
    if clients.len() <= 1 { return; }
    let message = serde_json::to_string(message).unwrap();
    for (key, other) in clients.iter() {
        if *key != *connection_id {
            other.send_raw(Message::from(message.clone())).await;
        }
    }
}
