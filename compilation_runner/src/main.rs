use tokio::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use anyhow::{Context};
use nix::libc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio_vsock::{OwnedReadHalf, OwnedWriteHalf};
use tracing::{error, info};
use fireframe_sdk::messaging::{MessageReader, MessageWriter};
use fireframe_sdk::types::{ClientMessage, ServerMessage};
use host_guest_protocol::{CommandResult, HostCommand};

const PROJECT_DIR: &str = "./project";

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    if !std::fs::exists("/dev/vsock").unwrap() {
        error!("vsock not available");
        return;
    }

    fs::create_dir_all(PROJECT_DIR).await.unwrap();

    let sock = tokio_vsock::VsockAddr::new(libc::VMADDR_CID_ANY, 1024);
    let listener = {
        info!("Setting up listener...");
        let mut l = tokio_vsock::VsockListener::bind(sock);
        while l.is_err() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            l = tokio_vsock::VsockListener::bind(sock);
        }
        l.unwrap()
    };

    info!("Listening on {}", sock);
    let mut running = Vec::new();
    while let Ok((stream, addr)) = listener.accept().await {
        info!("Accepting connection from {}", addr);
        // Firecracker sends OK <port>\n to host, we do nothing
        let (rx, tx) = stream.into_split();
        running.push(tokio::spawn(async move { serve_connection(rx, tx).await }));
    }

    for x in running {
        match x.await {
            Ok(x) => {
                if let Err(e) = x {
                    error!("connection error: {}", e);
                }
            }
            Err(e) => {
                error!("connection error: {}", e);
            }
        };
    }
}

async fn serve_connection(rx: OwnedReadHalf, tx: OwnedWriteHalf) -> anyhow::Result<()> {
    use host_guest_protocol::*;
    let mut reader = MessageReader::<OwnedReadHalf, ClientMessage<HostCommand>>::new(rx);
    let writer = MessageWriter::<OwnedWriteHalf, ServerMessage<CommandResult>>::new(tx);
    let writer = Arc::new(Mutex::new(writer));

    while let Ok(message) = reader.read().await {
        let writer = writer.clone();
        tokio::spawn(async move {
            info!("<<< {:#?}", message.request);
            if let Err(e) = handle_request(message, writer).await {
                error!("Error handling request: {}", e);
            }
        });
    }
    info!("Connection closed");
    Ok(())
}

async fn handle_request(message: ClientMessage<HostCommand>, writer: Arc<Mutex<MessageWriter<OwnedWriteHalf, ServerMessage<CommandResult>>>>) -> anyhow::Result<()>{
    match message.request {
        HostCommand::Run(script) => run(writer.clone(), message.id, script).await,
        HostCommand::ReadFile(path_arg) => {
            let path = PathBuf::from(PROJECT_DIR).join(&path_arg);
            let result = tokio::fs::read(path).await.ok();
            let result = CommandResult::FileContents(result);
            writer.lock().await.write(&ServerMessage {
                id: message.id,
                response: result,
            }).await.context("Failed to write response")?;
        }
        HostCommand::Clear => {
            _ = fs::remove_dir_all(PROJECT_DIR).await;
            if let Err(e) = fs::create_dir(PROJECT_DIR).await {
                writer.lock().await.write(&ServerMessage {
                    id: message.id,
                    response: CommandResult::Log(format!("Failed to recreate project dir: {:#}", e)),
                }).await.context("Failed to write response")?;
                return Ok(());
            }
            writer.lock().await.write(&ServerMessage {
                id: message.id,
                response: CommandResult::Completed,
            }).await.context("Failed to write response")?;
        }
        HostCommand::CreateFile { path, content } => {
            // FIXME: join and sanitize have surprising handling of absolute paths. Fix across all binaries
            let path = Path::new(PROJECT_DIR).join(sanitize_file_path(&path));
            if let Err(e) = tokio::fs::write(&path, content).await {
                writer.lock().await.write(&ServerMessage {
                    id: message.id,
                    response: CommandResult::Log(format!("Failed to write file: {:#}", e)),
                }).await.context("Failed to write response")?;
                return Ok(());
            }
            writer.lock().await.write(&ServerMessage {
                id: message.id,
                response: CommandResult::Completed,
            }).await.context("Failed to write response")?;
        }
        HostCommand::CreateDir { path } => {
            let path = Path::new(PROJECT_DIR).join(sanitize_file_path(&path));
            if let Err(e) = tokio::fs::create_dir_all(&path).await {
                writer.lock().await.write(&ServerMessage {
                    id: message.id,
                    response: CommandResult::Log(format!("Failed to create directory: {:#}", e)),
                }).await.context("Failed to write response")?;
                return Ok(());
            }
            writer.lock().await.write(&ServerMessage {
                id: message.id,
                response: CommandResult::Completed,
            }).await.context("Failed to write response")?;
        }
    }
    Ok(())
}


/// Returns a relative path without reserved names, redundant separators, ".", or "..".
fn sanitize_file_path(path: &str) -> PathBuf {
    // Replaces backwards slashes
    let path = path
        .replace('\\', "/")
        // Sanitizes each component
        .split('/')
        .map(sanitize_filename::sanitize)
        .collect::<String>();
    PathBuf::from(path)
}

async fn run<T: AsyncWriteExt + Unpin>(conn: Arc<Mutex<MessageWriter<T, ServerMessage<host_guest_protocol::CommandResult>>>>, id: u64, script: String) {
    use tokio::process::Command;
    use host_guest_protocol::*;

    let project_path = match PathBuf::from(PROJECT_DIR).canonicalize() {
        Ok(path) => path,
        Err(err) => {
            error!("Failed to get absolute path: {:#?}", err);
            PathBuf::from(PROJECT_DIR)
        }
    };
    let cmd = Command::new("/bin/busybox")
        .arg("sh")
        .arg("-c")
        .arg(script)
        .current_dir(PROJECT_DIR)
        .env("PROJECT_DIR", project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut cmd = match cmd {
        Ok(cmd) => cmd,
        Err(err) => {
            conn.lock()
                .await
                .write(&ServerMessage {
                    id,
                    response: CommandResult::Log(format!("Failed to recreate start process: {:#}", err)),
                })
                .await
                .unwrap();
            return;
        }
    };

    // Map output to socket
    let (tx, mut rx) = mpsc::channel(32);
    if let Some(stdout) = cmd.stdout.take() {
        relay_lines(stdout, tx.clone());
    }
    if let Some(stderr) = cmd.stderr.take() {
        relay_lines(stderr, tx.clone());
    }

    let status = loop {
        tokio::select! {
            res = cmd.wait() => {
                rx.close();
                break res;
            }
            Some(line) = rx.recv() => {
                conn.lock()
                    .await
                    .write(&ServerMessage {
                        id,
                        response: CommandResult::Log(line),
                    })
                    .await
                    .unwrap();
            }
        }
    }.unwrap();

    let code = status.code().unwrap();
    conn.lock()
        .await
        .write(&ServerMessage {
            id,
            response: CommandResult::ChildExit(code),
        })
        .await
        .unwrap();
}

fn relay_lines<R: tokio::io::AsyncRead + Unpin + Send + 'static>(readable: R, out: Sender<String>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut reader = BufReader::new(readable).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if let Err(_) = out.send(line).await { break }
        }
    })
}
