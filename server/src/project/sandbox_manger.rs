use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use fireframe_protocol::{FireframeCommand, FireframeResponse};
use fireframe_sdk::ipc::IpcClient;
use fireframe_sdk::messaging::{MessageReader, MessageWriter};
use tokio::net::{unix, UnixSocket};
use tokio::sync::RwLock;
use tracing::{error, info};
use crate::project::sandbox::{Sandbox};

pub(crate) struct SandboxManager {
    preloaded: RwLock<HashSet<String>>,
    running: RwLock<HashMap<String, Arc<Sandbox>>>,
    ipc_client: IpcClient<FireframeCommand, FireframeResponse, unix::OwnedReadHalf, unix::OwnedWriteHalf>,
}

impl SandboxManager {
    pub async fn new(socket_path: PathBuf) -> Result<SandboxManager, String> {
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

        Ok(SandboxManager {
            ipc_client: IpcClient::new(MessageReader::new(rx), MessageWriter::new(tx)),
            preloaded: Default::default(),
            running: Default::default(),
        })
    }

    pub async fn preload_vm(&self, tenant_id: &str) -> Result<(), String> {
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
                let sandbox = Sandbox::new(MessageReader::new(rx), MessageWriter::new(tx));

                assert!(!self.running.read().await.contains_key(tenant_id));
                self.running.write().await.insert(tenant_id.to_string(), Arc::new(sandbox));
                Ok(())
            },
            other => Err(format!("Unexpected or missing response: {other:?}")),
        }
    }

    pub async fn spawn(&self, tenant_id: &str) -> Result<Arc<Sandbox>, String> {
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
        let sandbox = Sandbox::new(MessageReader::new(rx), MessageWriter::new(tx));
        let sandbox = Arc::new(sandbox);
        self.running.write().await.insert(tenant_id.to_string(), sandbox.clone());

        Ok(sandbox)
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

    pub async fn kill_vm(&self, tenant_id: &str) -> Result<(), String> {
        self.running.write().await.remove(tenant_id);
        let result = self.ipc_client.call(FireframeCommand::KillVM {tenant_id: tenant_id.to_string()}).await;
        match result {
            Ok(FireframeResponse::VMKilled) | Ok(FireframeResponse::VMNotRunning) => Ok(()),
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

    /// Removes all VMs
    #[allow(dead_code, reason = "Implement admin button #33")]
    pub async fn reset(&self) -> Result<(), String> {
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
