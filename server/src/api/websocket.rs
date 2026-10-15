use crate::api::types::{self, ProjectId, WsClientCompile, WsClientDocUpdate, WsClientMessage, WsClientSetAsset, WsServerAssetUpdated, WsServerAssetUpdatedT, WsServerAwareness, WsServerAwarenessT, WsServerDocUpdate, WsServerDocUpdateT, WsServerLog, WsServerMessage, WsServerResetLog, WsServerSetPdf};
use crate::project::sandbox_manager::{SandboxManager};
use crate::project::{Project};
use crate::settings::ServerSettings;
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use futures::{FutureExt, TryFutureExt};
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::io::DuplexStream;
use tokio::sync::{mpsc};
use tracing::{error, trace, warn};
use crate::project::zip_fs::ZipFS;

const PING_TIMEOUT: Duration = Duration::from_secs(30);
const CLOSE_WEBSOCKET_PROJECT_TIMEOUT: Duration = Duration::from_secs(40);

#[derive(Clone)]
pub struct WebsocketProject<S: SandboxManager> {
    clients: broadcast::BroadcastActorHandle,
    inner: Arc<Project<S>>,
}

impl<S: SandboxManager> WebsocketProject<S> {
    pub async fn load(
        db: SqlitePool,
        project_id: ProjectId,
        sandbox_manager: Arc<S>,
        on_disconnected: tokio::sync::oneshot::Sender<ProjectId>,
        settings: Arc<ServerSettings>,
    ) -> Self {
        let (tx, mut rx) = tokio::sync::watch::channel(0);
        // send closed event, if no-one connected for 30 s
        tokio::spawn(async move {
            loop {
                if *rx.borrow() != 0 {
                    if rx.wait_for(|v| *v == 0).await.is_err() { break };
                }
                let stays_disconnected = tokio::time::timeout(CLOSE_WEBSOCKET_PROJECT_TIMEOUT, rx.wait_for(|v| *v != 0)).await;

                if let Ok(value) = stays_disconnected {
                    if value.is_err() { break };
                } else {
                    break; // No new connection for timeout
                }
            }

            debug_assert!(*rx.borrow() == 0);
            _ = on_disconnected.send(project_id);
        });
        WebsocketProject {
            clients: broadcast::start(tx),
            inner: Arc::new(Project::new(db, project_id, sandbox_manager, settings).await),
        }
    }

    #[cfg(test)]
    pub fn get_project(&self) -> Arc<Project<S>> {
        self.inner.clone()
    }

    pub fn kick(&self, user: types::UserId) -> impl Future<Output = ()> {
        self.clients.kick(user)
    }

    pub async fn connect(&self, ws: WebSocket, user: types::UserId) {
        let (sink, stream) = ws.split();
        let (actor, client) = SendActor::new(sink);
        tokio::spawn(actor.start());
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
        let cancellation_token = CancellationToken::new();
        self.clients.register(user, client.clone(), cancellation_token.clone()).await;

        // Ensure regular pings
        let pong_received_clone = pong_received.clone();
        let clients = self.clients.clone();
        let ping_client = client.clone();
        let interval = tokio::task::spawn(async move {
            let mut interval = tokio::time::interval(PING_TIMEOUT);
            loop {
                interval.tick().await;
                if !pong_received_clone.load(Ordering::Relaxed) {
                    clients.kick(user).await;
                    break;
                }
                pong_received_clone.store(false, Ordering::Relaxed);
                ping_client.send_raw(Message::Ping(Bytes::new())).await;
            }
        });

        let handle = stream.for_each_concurrent(10, async |message| {
            if cancellation_token.is_cancelled() { return; }
            let Ok(message) = message else { return; };
            match message {
                Message::Text(raw_message) => {
                    pong_received.fetch_or(true, Ordering::Relaxed);
                    let Ok(msg) = serde_json::from_str::<WsClientMessage>(&raw_message)
                    else {
                        warn!("Received unexpected message: {:?}", raw_message);
                        return;
                    };
                    self.handle_message(msg, user, client.clone()).await;
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
                    self.clients.deregister(user).await;
                }
            };
        });

        tokio_util::future::FutureExt::with_cancellation_token(handle, &cancellation_token).await;

        interval.abort();
    }

    async fn handle_message(&self, msg: WsClientMessage, user: types::UserId, client: SendActorHandle) {
        match msg {
            WsClientMessage::DocUpdate (WsClientDocUpdate {
                update: encoded_update, ..
            }) => {
                let Ok(update) = BASE64_STANDARD.decode(&encoded_update) else {
                    warn!("Received invalid base64 data");
                    return;
                };
                self.inner.crdt.apply_update(&update, user).await;
                self.clients.send_to_all_except(user, &WsServerMessage::DocUpdate(WsServerDocUpdate {
                    t: WsServerDocUpdateT::U,
                    update: encoded_update
                })).await;
            }
            WsClientMessage::Awareness(update) => {
                self.clients.send_to_all_except(user, &WsServerMessage::Awareness(WsServerAwareness {
                    t: WsServerAwarenessT::A, update: update.update
                })).await;
            }
            WsClientMessage::SetAsset(WsClientSetAsset { uuid, data, .. })  => {
                let Ok(data) = BASE64_STANDARD.decode(data) else {
                    warn!("Received invalid base64 data");
                    return;
                };
                self.inner.crdt.set_asset(uuid.clone(), data, user).await;
                self.clients.send_to_all_except(user, &WsServerMessage::AssetUpdated(WsServerAssetUpdated {
                    t: WsServerAssetUpdatedT::AssetUpdated, uuid
                })).await;
            }
            WsClientMessage::Compile(WsClientCompile{ uuid, compiler, .. }) => {
                let (tx, mut rx) = mpsc::channel(4);
                let compilation = self.inner.compile(uuid.clone(), compiler.into(), tx.clone());
                let log_handle = {
                    let client = client.clone();
                    let uuid = uuid.clone();
                    tokio::spawn(async move {
                        while let Some(msg) = rx.recv().await {
                            client.send(&WsServerMessage::Log(WsServerLog {
                                t: types::WsServerLogT::Log,
                                uuid: uuid.clone(),
                                data: msg,
                            })).await;
                        }
                        trace!("logging channel closed");
                    })
                };
                client.send(&WsServerMessage::ResetLog(WsServerResetLog {
                    t: types::WsServerResetLogT::ResetLog,
                    uuid: uuid.clone(),
                })).await;
                match compilation.await {
                    Err(e) => {
                        _ = tx.send(format!("Execution error: {}", e)).await;
                    }
                    Ok(result) => {
                        // Broadcast successful result
                        if let Some(doc) = result.document {
                            let pdf = format!(
                                "data:application/pdf;base64,{}",
                                BASE64_STANDARD.encode(doc)
                            );
                            self.clients.broadcast(&WsServerMessage::SetPdf(WsServerSetPdf {
                                t: types::WsServerSetPdfT::SetPdf,
                                uuid,
                                content: Some(pdf),
                            })).await;
                        } else {
                            client.send(&WsServerMessage::SetPdf(WsServerSetPdf {
                                t: types::WsServerSetPdfT::SetPdf,
                                uuid,
                                content: None,
                            })).await;
                        }
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

struct SendActor {
    sink: SplitSink<WebSocket, Message>,
    receiver: mpsc::Receiver<Message>,
}

impl SendActor {
    pub fn new(sink: SplitSink<WebSocket, Message>) -> (SendActor, SendActorHandle) {
        let (tx, rx) = mpsc::channel(32);
        let handle = SendActorHandle(tx);
        let actor = SendActor {
            sink,
            receiver: rx,
        };
        (actor, handle)
    }

    async fn start(mut self) {
        while let Some(message) = self.receiver.recv().await {
            if let Err(err) = self.sink.send(message).await {
                warn!("Unable to send message: {}", err);
            }
        }
    }
}

#[derive(Clone)]
struct SendActorHandle(mpsc::Sender<Message>);

impl SendActorHandle {
    pub fn send(&self, message: &WsServerMessage) -> impl Future<Output = ()> {
        let serialized = serde_json::to_string(message).unwrap();
        let serialized = Message::from(serialized);
        self.send_raw(serialized)
    }

    pub fn send_raw(&self, message: Message) -> impl Future<Output = ()> {
        self.0.send(message)
            .inspect_err(|e| error!("Unable to send message to client: {:?}", e))
            .map(|_| ())
    }
}

mod broadcast {
    use std::{collections::HashMap};

    use axum::extract::ws::Message;
    use futures::{FutureExt, TryFutureExt};
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;
use tracing::error;

    use crate::api::{types::{self, WsServerMessage}, websocket::SendActorHandle};

    enum BroadcastActorMessage {
        Register(types::UserId, SendActorHandle, CancellationToken),
        Deregister(types::UserId),
        Kick(types::UserId),
        SendToAllExcept(Option<types::UserId>, Message),
    }

    struct BroadcastActor {
        receiver: mpsc::Receiver<BroadcastActorMessage>,
        connected: HashMap<types::UserId, SendActorHandle>,
        cancel_tokens: HashMap<types::UserId, CancellationToken>,
        user_count: tokio::sync::watch::Sender<usize>,
    }

    impl BroadcastActor {
        pub async fn spawn(mut self) {
            while let Some(msg) = self.receiver.recv().await {
                self.handle_message(msg).await;
            }
        }

        async fn handle_message(&mut self, msg: BroadcastActorMessage) {
            match msg {
                BroadcastActorMessage::Register(user_id, send_actor_handle, cancel_token) => {
                    _ = self.connected.insert(user_id, send_actor_handle);
                    _ = self.cancel_tokens.insert(user_id, cancel_token);
                    self.user_count.send_replace(self.connected.len());
                },
                BroadcastActorMessage::Deregister(user_id) => {
                    _ = self.connected.remove(&user_id);
                    _ = self.cancel_tokens.remove(&user_id);
                    self.user_count.send_replace(self.connected.len());
                },
                BroadcastActorMessage::Kick(user_id) => {
                    let Some(token) = self.cancel_tokens.remove(&user_id) else { return };
                    token.cancel();
                    let removed = self.connected.remove(&user_id).expect("cancel_tokens element exists without connected");
                    removed.send_raw(Message::Close(None)).await;
                    self.user_count.send_replace(self.connected.len());
                },
                BroadcastActorMessage::SendToAllExcept(user_id, message) => self.send_to_all_except(user_id, message).await,
            }
        }

        fn send_to_all_except(&self, user_id: Option<types::UserId>, message: Message) -> impl Future<Output = ()> {
            let requests = self.connected.iter()
                .filter(|(id, _)| user_id.is_none_or(|uid| **id != uid))
                .map(|(_, handle)| handle.send_raw(message.clone()));
            futures::future::join_all(requests).map(|_| ())
        }
    }

    pub fn start(user_count: tokio::sync::watch::Sender<usize>) -> BroadcastActorHandle {
        let (tx, rx) = mpsc::channel(64);
        let handle = BroadcastActorHandle(tx);
        let actor = BroadcastActor {
            connected: HashMap::new(),
            cancel_tokens: HashMap::new(),
            receiver: rx,
            user_count: user_count,
        };
        tokio::spawn(actor.spawn());
        handle
    }

    #[derive(Clone)]
    pub struct BroadcastActorHandle(mpsc::Sender<BroadcastActorMessage>);

    impl BroadcastActorHandle {
        pub fn register(&self,
            user_id: types::UserId,
            handle: SendActorHandle,
            cancel_token: CancellationToken,
        ) -> impl Future<Output = ()> {
            self.send(BroadcastActorMessage::Register(user_id, handle, cancel_token))
        }

        pub fn deregister(&self, user_id: types::UserId) -> impl Future<Output = ()> {
            self.send(BroadcastActorMessage::Deregister(user_id))
        }

        pub fn kick(&self, user_id: types::UserId) -> impl Future<Output = ()> {
            self.send(BroadcastActorMessage::Kick(user_id))
        }

        pub fn broadcast(&self, message: &WsServerMessage) -> impl Future<Output = ()> {
            let message = serde_json::to_string(message).unwrap();
            self.send(BroadcastActorMessage::SendToAllExcept(None, Message::from(message)))
        }

        pub fn send_to_all_except(&self, user_id: types::UserId, message: &WsServerMessage) -> impl Future<Output = ()> {
            let message = serde_json::to_string(message).unwrap();
            self.send(BroadcastActorMessage::SendToAllExcept(Some(user_id), Message::from(message)))
        }

        fn send(&self, msg: BroadcastActorMessage) -> impl Future<Output = ()> {
            self.0.send(msg)
                .inspect_err(|e| error!("Dropped BroadcastActorMessage: {}", e))
                .map(|_| ())
        }
    }
}
