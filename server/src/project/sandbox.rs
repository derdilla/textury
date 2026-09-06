use fireframe_sdk::ipc::IpcClient;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{mpsc};
use tracing::{error, info, warn};
use crate::project::filesystem::Filesystem;

pub struct Sandbox {
    pub ipc_client: IpcClient<host_guest_protocol::HostCommand, host_guest_protocol::CommandResult, OwnedReadHalf, OwnedWriteHalf>,
}

impl Sandbox {
    pub fn new(reader: fireframe_sdk::messaging::MessageReader<OwnedReadHalf,
                    fireframe_sdk::types::ServerMessage<host_guest_protocol::CommandResult>>,
               writer: fireframe_sdk::messaging::MessageWriter<OwnedWriteHalf,
                    fireframe_sdk::types::ClientMessage<host_guest_protocol::HostCommand>>) -> Self {
        Self {
            ipc_client: IpcClient::new(reader, writer),
        }
    }

    /// Run a shell command in the sandboxed filesystem.
    /// Returns the exit code of the command, if known.
    pub async fn run(&self, script: String, logger_tx: Option<mpsc::Sender<String>>) -> Option<i32> {
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

    pub async fn read_file(&self, path: String) -> Option<Vec<u8>> {
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

impl Filesystem for Sandbox {
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
