use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use fireframe_protocol::{FireframeCommand, FireframeResponse};
use fireframe_sdk::ipc::IpcClient;
use fireframe_sdk::messaging::{MessageReader, MessageWriter};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{unix, UnixSocket};
use tokio::sync::{RwLock, mpsc};
use tracing::{error, info, warn};
use crate::project::filesystem::Filesystem;
use crate::project::sandbox::Sandbox;
use crate::project::sandbox_manager::SandboxManager;

pub struct FireframeManager {
    preloaded: RwLock<HashSet<String>>,
    running: RwLock<HashMap<String, Arc<FireframeSandbox>>>,
    ipc_client: IpcClient<FireframeCommand, FireframeResponse, unix::OwnedReadHalf, unix::OwnedWriteHalf>,
}

impl FireframeManager {
    pub async fn new(socket_path: PathBuf) -> Result<FireframeManager, String> {
        // We can't spawn fireframe, because it needs root.
        // So we need to connect to an existing fireframe.

        if !socket_path.exists() {
            return Err("No firecracker socket not found".to_string());
        }

        let socket = UnixSocket::new_stream()
            .map_err(|e| format!("Unable to create socket: {e:?}"))?;
        let stream = socket.connect(&socket_path).await
            .map_err(|e| format!("Unable to connect to socket: {e:?}"))?;
        let (rx, tx) = stream.into_split();

        Ok(FireframeManager {
            ipc_client: IpcClient::new(MessageReader::new(rx), MessageWriter::new(tx)),
            preloaded: Default::default(),
            running: Default::default(),
        })
    }

    async fn start_vm(&self, tenant_id: &str) -> Result<PathBuf, String> {
        info!("Starting VM for {tenant_id}");
        let result = self.ipc_client.call(FireframeCommand::StartVM {tenant_id: tenant_id.to_string()}).await;
        match result {
            Ok(FireframeResponse::VMStarted { socket_path }) => Ok(socket_path),
            Ok(FireframeResponse::VMAlreadyRunning { socket_path }) => Ok(socket_path),
            Ok(FireframeResponse::VMNotSetUp) => {
                error!("Called StartVM before SetupVM");
                Err("VM not setup up".to_string())
            },
            other => Err(format!("Unexpected or missing response: {other:?}")),
        }
    }

    #[allow(dead_code, reason = "Don't forget we can do this")]
    async fn delete_vm(&self, tenant_id: &str) -> Result<(), String> {
        let result = self.ipc_client.call(FireframeCommand::DeleteVM {tenant_id: tenant_id.to_string()}).await;
        match result {
            Ok(FireframeResponse::VMRemoved) => {
                self.preloaded.write().await.remove(tenant_id);
                self.running.write().await.remove(tenant_id);
                Ok(())
            },
            other => Err(format!("Unexpected or missing response: {other:?}")),
        }
    }
}

impl SandboxManager for FireframeManager {
    type Sandbox = FireframeSandbox;

    async fn preload_vm(&self, tenant_id: &str) -> Result<(), String> {
        if self.preloaded.read().await.contains(tenant_id) {
            return Ok(());
        }
        info!("Setting up VM snapshot for {tenant_id}");
        let result = tokio::time::timeout(Duration::from_secs(10),
            self.ipc_client.call(FireframeCommand::SetupSnapshot {tenant_id: tenant_id.to_string()})).await;

        match result {
            Ok(Ok(FireframeResponse::SnapshotCreated)) | Ok(Ok(FireframeResponse::SnapshotAlreadyExists)) => {
                // workaround: https://codeberg.org/derdilla/fireframe/issues/3
                tokio::time::sleep(Duration::from_millis(200)).await;
                self.preloaded.write().await.insert(tenant_id.to_string());
                Ok(())
            },
            Ok(Ok(FireframeResponse::VMAlreadyRunning { socket_path })) => {
                self.preloaded.write().await.insert(tenant_id.to_string());
                let (rx, tx) = fireframe_sdk::messaging::connect_to_vm(socket_path).await
                    .map_err(|e| format!("Unable to connect to VM: {e:?}"))?;
                let sandbox = FireframeSandbox::new(MessageReader::new(rx), MessageWriter::new(tx));

                assert!(!self.running.read().await.contains_key(tenant_id));
                self.running.write().await.insert(tenant_id.to_string(), Arc::new(sandbox));
                Ok(())
            },
            other => Err(format!("Unexpected or missing response: {other:?}")),
        }
    }

    async fn spawn(&self, tenant_id: &str) -> Result<Arc<FireframeSandbox>, String> {
        if let Some(vm) = self.running.read().await.get(tenant_id) {
            return Ok(vm.clone());
        }

        if let Err(e) = self.preload_vm(tenant_id).await {
            error!("preload_vm failed: {e:?}");
            return Err(e);
        }
        let socket = match self.start_vm(tenant_id).await {
            Ok(socket) => socket,
            Err(e) => {
                error!("start_vm failed: {e:?}");
                return Err(e);
            },
        };
        // workaround: https://codeberg.org/derdilla/fireframe/issues/3
        tokio::time::sleep(Duration::from_millis(300)).await;


        let (rx, tx) = fireframe_sdk::messaging::connect_to_vm(socket).await
            .map_err(|e| format!("Unable to connect to VM: {e:?}"))?;
        let sandbox = FireframeSandbox::new(MessageReader::new(rx), MessageWriter::new(tx));
        let sandbox = Arc::new(sandbox);
        self.running.write().await.insert(tenant_id.to_string(), sandbox.clone());

        Ok(sandbox)
    }

    async fn kill_vm(&self, tenant_id: &str) -> Result<(), String> {
        self.running.write().await.remove(tenant_id);
        let result = self.ipc_client.call(FireframeCommand::KillVM {tenant_id: tenant_id.to_string()}).await;
        match result {
            Ok(FireframeResponse::VMKilled) | Ok(FireframeResponse::VMNotRunning) => Ok(()),
            other => Err(format!("Unexpected or missing response: {other:?}")),
        }
    }

    async fn reset(&self) -> Result<(), String> {
        let result = self.ipc_client.call(FireframeCommand::Reset).await;
        match result {
            Ok(FireframeResponse::ResetSuccessful) => {
                self.preloaded.write().await.clear();
                self.running.write().await.clear(); // Sandboxes will follow with broken pipe wave
                Ok(())
            },
            other => Err(format!("Unexpected or missing response: {other:?}")),
        }
    }

}

pub struct FireframeSandbox {
    pub ipc_client: IpcClient<host_guest_protocol::HostCommand, host_guest_protocol::CommandResult, OwnedReadHalf, OwnedWriteHalf>,
}

impl FireframeSandbox {
    pub(self) fn new(reader: fireframe_sdk::messaging::MessageReader<OwnedReadHalf,
                    fireframe_sdk::types::ServerMessage<host_guest_protocol::CommandResult>>,
               writer: fireframe_sdk::messaging::MessageWriter<OwnedWriteHalf,
                    fireframe_sdk::types::ClientMessage<host_guest_protocol::HostCommand>>) -> Self {
        Self {
            ipc_client: IpcClient::new(reader, writer),
        }
    }
}

impl Sandbox for FireframeSandbox {
    async fn run(&self, script: String, logger_tx: Option<mpsc::Sender<String>>) -> Option<i32> {
        let (tx, mut rx) = mpsc::channel(8);
        if let Err(e) = self.ipc_client.subscribe(host_guest_protocol::HostCommand::Run(script), tx).await {
            error!("Failed to subscribe to run command: {e:#?}");
            return None
        }

        let mut res = None;
        while let Some(resp) = rx.recv().await {
            match resp {
                host_guest_protocol::CommandResult::ChildExit(code) => {
                    res = Some(code);
                    break
                }
                host_guest_protocol::CommandResult::Log(msg) => {
                    if let Some(logger_tx) = &logger_tx {
                        if let Err(e) = logger_tx.send(msg).await {
                            error!("Failed to forward log message: {}", e);
                        }
                    }
                }
                resp => {
                    warn!("Unexpected response for run command: {resp:#?}");
                }
            }
        };

        res
    }

    async fn read_file(&self, path: String) -> Option<Vec<u8>> {
        let path = non_absolute(path);
        info!("HostCommand::ReadFile({path})");
        let req = self.ipc_client.call(host_guest_protocol::HostCommand::ReadFile(path));
        match req.await {
            Ok(host_guest_protocol::CommandResult::FileContents(content)) => content,
            res => {
                error!("Unexpected response for read_file: {res:#?}");
                None
            }
        }
    }
}

impl Filesystem for FireframeSandbox {
    async fn create_file(&self, path: String, content: &Vec<u8>) -> Result<(), String> {
        info!("create_file({})", &path);
        let path = non_absolute(path);
        let response = self.ipc_client.call(host_guest_protocol::HostCommand::CreateFile {
            path,
            content: content.clone(),
        }).await;

        match response {
            Ok(host_guest_protocol::CommandResult::Completed) => Ok(()),
            resp => Err(format!("Failed to create file, got: {resp:#?}")),
        }
    }

    async fn create_dir(&self, path: String) -> Result<(), String> {
        info!("create_dir({})", &path);
        let path = non_absolute(path);
        let response = self.ipc_client.call(host_guest_protocol::HostCommand::CreateDir {
            path,
        }).await;

        match response {
            Ok(host_guest_protocol::CommandResult::Completed) => Ok(()),
            resp => Err(format!("Failed to create file, got: {resp:#?}")),
        }
    }
}

fn non_absolute(path: String) -> String {
    path.trim_start_matches('/').to_string()
}
