use crate::api::types::ProjectId;
use crate::compilation::{CompilationResult, Compiler};
use crate::project::db_synced_crdt::DbSyncedCRDT;
use sqlx::SqlitePool;
use tracing::debug;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;

use crate::project::sandbox_manager::SandboxManager;

pub mod db_synced_crdt;
pub mod filesystem;
pub mod zip_fs;

pub mod sandbox_manager;
pub mod sandbox;
#[cfg(feature = "fireframe")]
pub mod fireframe_manager;
pub mod local_sandbox_manager;
pub mod disk_fs;

pub struct Project<S: SandboxManager> {
    pub db: SqlitePool,
    pub crdt: DbSyncedCRDT,
    pub connections: AtomicU32,
    sandbox_manager: Arc<S>,
}

impl<S: SandboxManager> Project<S> {
    pub async fn new(
        db: SqlitePool,
        id: ProjectId,
        doc: y_octo::Doc,
        sandbox_manager: Arc<S>,
    ) -> Self {
        Project {
            db: db.clone(),
            crdt: DbSyncedCRDT::new(db, id, doc),
            connections: Default::default(),
            sandbox_manager,
        }
    }

    pub async fn compile(&self, uuid: String, compiler: Compiler) -> Result<CompilationResult, String> {
        use tokio::sync::mpsc;
        use tracing::{error};

        use crate::project::sandbox::Sandbox;

        let Some(initial_path) = self.crdt.lookup_uuid(&uuid) else {
            return Err(format!("No file for {uuid}"));
        };

        let id = self.crdt.id();
        let Some(script) = compiler.build_remote_script(&format!("{initial_path}")) else {
            return Err("Unable to build script".to_string());
        };
        debug!("Running script: {}", &script);

        let sandbox = match self.sandbox_manager.spawn(&id.to_string()).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed to spawn sandbox: {e}")),
        };
        if let Err(e) = self.crdt.write_to_fs(sandbox.as_ref()).await {
            if let Err(e) = self.sandbox_manager.kill_vm(&id.to_string()).await {
                error!("Unable to kill vm: {e}");
            }
            return Err(format!("Error: Unable to write to fs: {e}"));
        }
        let (tx, mut rx) = mpsc::channel(8);
        let log_handle = tokio::spawn(async move {
            use tracing::trace;

            let mut log = vec![];
            while let Some(msg) = rx.recv().await {
                trace!("<<< {msg}");
                log.push(msg);
            }
            trace!("logging channel closed");
            log.join("\n")
        });
        let return_value = sandbox.as_ref().run(script, Some(tx)).await;

        let document = sandbox.as_ref().read_file("textury_out.pdf".to_string()).await;
        if let Err(e) = self.sandbox_manager.kill_vm(&id.to_string()).await {
            error!("Unable to kill vm: {e}");
        }

        let log = if let Some(exit_code) = return_value {
            let mut log = log_handle.await.unwrap_or("Unable to collect log: JoinError".to_string());
            log.push_str(&format!("\nProcess returned with exit code: {}", exit_code));
            log
        } else {
            "Unable to start process".to_string()
        };

        self.update_latest_compilation(&document).await;

        Ok(CompilationResult {
            log,
            document,
        })
    }

    async fn update_latest_compilation(&self, document: &Option<Vec<u8>>) {
        if document.is_none() {
            return;
        };
        let id = self.crdt.id();
        sqlx::query!(
            "UPDATE projects SET latest_compilation = ? WHERE id = ?",
            document, id)
            .execute(&self.db).await
            .unwrap();
    }
}
