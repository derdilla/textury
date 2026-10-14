use std::{collections::HashMap, os::unix::process::ExitStatusExt, process::Stdio, sync::{Arc, atomic::AtomicBool}};

use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command, sync::{RwLock, mpsc::Sender}, task::JoinHandle};
use tracing::{error, warn};

use crate::project::{disk_fs::DiskFS, filesystem::Filesystem, sandbox::Sandbox, sandbox_manager::SandboxManager};

pub struct LocalSandboxManager {
    sandboxes: RwLock<HashMap<String, Arc<LocalSandbox>>>,
    has_bwrap: bool
}

impl LocalSandboxManager {
    pub async fn new() -> Self {
        let has_bwrap = Command::new("bwrap")
            .args(["--unshare-all", "--die-with-parent", "--ro-bind", "/", "/"])
            .arg("true")
            .stdout(Stdio::null()).stderr(Stdio::null())
            .output().await
            .is_ok_and(|res| res.status.success());
        LocalSandboxManager { sandboxes: Default::default(), has_bwrap }
    }
}

impl SandboxManager for LocalSandboxManager {
    type Sandbox = LocalSandbox;

    async fn preload_vm(&self, tenant_id: &str) -> Result<(), String> {
        if !self.sandboxes.read().await.contains_key(tenant_id) {
            self.sandboxes.write().await
                .insert(tenant_id.to_string(), Arc::new(LocalSandbox::new(self.has_bwrap).await));
        }
        Ok(())
    }

    async fn spawn(&self, tenant_id: &str) -> Result<std::sync::Arc<Self::Sandbox>, String> {
        self.preload_vm(tenant_id).await?;
        let sandbox = self.sandboxes.read().await
            .get(tenant_id).expect("Sandbox was just created").clone();
        Ok(sandbox)
    }

    async fn kill_vm(&self, tenant_id: &str) -> Result<(), String> {
        if let Some(sb) = self.sandboxes.write().await.remove(tenant_id) {
            sb.kill();
        };
        Ok(())
    }
}

pub struct LocalSandbox {
    fs: DiskFS,
    has_bwrap: bool,
    is_dead: AtomicBool,
}

impl LocalSandbox {
    pub(self) async fn new(has_bwrap: bool) -> LocalSandbox {
        let dir = tempfile::tempdir().unwrap();
        LocalSandbox {
            fs: DiskFS::new(dir.keep()),
            has_bwrap: has_bwrap,
            is_dead: AtomicBool::new(false),
        }
    }

    pub fn kill(&self) {
        self.is_dead.store(true, std::sync::atomic::Ordering::Relaxed);
        // TODO: kill running children
        let root = self.fs.root();
        _ = std::fs::remove_dir_all(root);
    }
}

impl Sandbox for LocalSandbox {
    async fn run(&self, script: String, logger_tx: Option<tokio::sync::mpsc::Sender<String>>) -> Option<i32> {
        if self.is_dead.load(std::sync::atomic::Ordering::Relaxed) {
            warn!("Tried to use killed VM");
            log(logger_tx, "Tried to use killed VM").await;
            return None;
        }

        let path = std::env::var("PATH")
            .unwrap_or("/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/texlive/bin/x86_64-linuxmusl".to_string());

        #[cfg(not(feature = "alpine"))]
        let bin_dirs = [
            "--symlink", "usr/lib", "/lib",
            "--symlink", "usr/lib64", "/lib64",
            "--symlink", "usr/bin", "/bin",
            "--symlink", "usr/sbin", "/sbin",
        ];
        #[cfg(feature = "alpine")]
        let bin_dirs = [
            "--ro-bind-try", "/lib", "/lib",
            "--ro-bind-try", "/sbin", "/sbin",
            "--ro-bind-try", "/bin", "/bin",
        ];

        let child = if self.has_bwrap {
            const PROJECT_DIR: &'static str = "/project";
            Command::new("bwrap")
                .args(["--ro-bind", "/usr", "/usr"])
                .args(["--tmpfs", "/tmp"])
                .args(["--dir", "/var"])
                .args(["--ro-bind-try", "/var/lib/texmf", "/var/lib/texmf"])
                .args(["--ro-bind-try", "/opt", "/opt"]) // /opt/texlive
                .args(["--dir", "/user"])
                .args(["--symlink", "../tmp", "/var/tmp"])
                .args(["--proc", "/proc"])
                .args(["--dev", "/dev"])
                .args(["--ro-bind", "/etc/resolv.conf", "/etc/resolv.conf"])
                .args(bin_dirs)
                .arg("--unshare-all")
                .arg("--share-net")
                .arg("--die-with-parent")
                .arg("--clearenv")
                .arg("--bind").arg(self.fs.root()).arg(PROJECT_DIR)
                .arg("--setenv").arg("PROJECT_DIR").arg(PROJECT_DIR)
                .arg("--setenv").arg("PATH").arg(path)
                .arg("--chdir").arg("/user")
                .arg("/bin/sh")
                .arg("-c")
                .arg(script)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        } else {
            warn!("Compiling without sandboxing");
            Command::new("sh")
                .env_clear()
                .env("PROJECT_DIR", self.fs.root())
                .env("PATH", path)
                .current_dir(self.fs.root())
                .arg("-c")
                .arg(script)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        };

        let mut child = match child {
            Ok(child) => child,
            Err(err) => {
                log(logger_tx, format!("Unable to start child: {}", err)).await;
                return None;
            },
        };

        if let Some(logger_tx) = logger_tx {
            if let Some(stdout) = child.stdout.take() {
                relay_lines(stdout, logger_tx.clone());
            }
            if let Some(stdout) = child.stderr.take() {
                relay_lines(stdout, logger_tx);
            }
        }

        child.wait().await.ok().map(|e| e.into_raw())
    }

    async fn read_file(&self, path: String) -> Option<Vec<u8>> {
        if self.is_dead.load(std::sync::atomic::Ordering::Relaxed) {
            warn!("Tried to use killed VM");
        }
        let path = self.fs.validated_path(path).ok()?;
        tokio::fs::read(path).await.ok()
    }
}

async fn log<M: ToString>(logger: Option<tokio::sync::mpsc::Sender<String>>, msg: M) {
    if let Some(logger) = logger {
        if let Err(e) = logger.send(msg.to_string()).await {
            error!("Failed to send log message: {}", e)
        }
    }
}

fn relay_lines<R: tokio::io::AsyncRead + Unpin + Send + 'static>(readable: R, out: Sender<String>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut reader = BufReader::new(readable).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if let Err(_) = out.send(line).await { break }
        }
    })
}

impl Filesystem for LocalSandbox {
    fn create_file(&self, path: String, content: &Vec<u8>) -> impl Future<Output = Result<(), String>> {
        if self.is_dead.load(std::sync::atomic::Ordering::Relaxed) {
            warn!("Tried to use killed VM");
        }
        self.fs.create_file(path, content)
    }

    fn create_dir(&self, path: String) -> impl Future<Output = Result<(), String>> {
        if self.is_dead.load(std::sync::atomic::Ordering::Relaxed) {
            warn!("Tried to use killed VM");
        }
        self.fs.create_dir(path)
    }
}

impl Drop for LocalSandbox {
    fn drop(&mut self) {
        self.kill();
    }
}
