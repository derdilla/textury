use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tarpc::tokio_serde::formats::Bincode;
use tempfile::TempDir;
use tokio::net::UnixStream;
use worker_api::WorkerServiceClient;

const WORKER_PATH: &str = "/home/derdilla/code/dillaleaf/target/debug/worker";

#[derive(Default)]
pub struct WorkerManager {
    running_workers: Vec<WorkerInfo>,
}

impl WorkerManager {
    async fn start_worker(&mut self, repository: String) -> Result<PathBuf, String> {
        debug_assert!(
            self.running_workers
                .iter()
                .find(|e| e.repository == repository)
                .is_none()
        );

        let dir = tempfile::tempdir().map_err(|err| err.to_string())?;
        println!("Worker directory: {}", dir.path().display());

        let repo_path = dir.path().join("repo");
        let result = Command::new("git")
            .args(["clone", &repository, repo_path.to_str().unwrap()])
            .output()
            .map_err(|err| err.to_string())?;
        if !result.status.success() {
            return Err(String::from_utf8(result.stderr).unwrap());
        }

        let socket_path = dir.path().join("worker.sock");
        //fs::write(&socket_path, "").map_err(|err| err.to_string())?;
        //let stream = UnixStream::connect(&socket_path).await.map_err(|err| err.to_string())?;

        Command::new(WORKER_PATH)
            .args([socket_path.to_str().unwrap(), repo_path.to_str().unwrap()])
            .spawn()
            .map_err(|err| err.to_string())?;

        self.running_workers.push(WorkerInfo {
            repository,
            root: repo_path.to_str().unwrap().to_string(),
            socket: socket_path.clone(),
            listeners: 1,
            dir,
        });

        // Wait for child to launch
        let res = tokio::time::timeout(tokio::time::Duration::from_secs(2), async {
            while !socket_path.exists() {
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }
        })
        .await;
        if res.is_err() {
            return Err("Worker didn't start in time".into());
        }

        Ok(socket_path)
    }

    pub async fn get_worker(&mut self, repository: String) -> Result<WorkerServiceClient, String> {
        let socket = self
            .running_workers
            .iter_mut()
            .find(|e| e.repository == repository);
        let socket = match socket {
            None => self.start_worker(repository).await?,
            Some(s) => {
                s.listeners += 1; // todo: decrement somewhere and dispose when 0
                s.socket.clone()
            }
        };

        let transport = tarpc::serde_transport::unix::connect(socket, Bincode::default)
            .await
            .map_err(|e| format!("Failed to connect to worker: '{}'", e.to_string()))?;

        // Max 200 MB messages, same as worker
        //transport.config_mut().max_frame_length(200 * 1024 * 1024);

        Ok(WorkerServiceClient::new(tarpc::client::Config::default(), transport).spawn())
    }
}

struct WorkerInfo {
    repository: String,
    root: String,
    socket: PathBuf,
    listeners: usize,
    // To drop later
    dir: TempDir,
}

// TODO: don't delete uncommited changes
