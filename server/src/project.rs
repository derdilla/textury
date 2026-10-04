use crate::api::types::ProjectId;
use crate::compilation::{CompilationResult, Compiler};
use crate::project::db_synced_crdt::DbSyncedCRDT;
use sqlx::SqlitePool;
#[cfg(feature = "fireframe")]
use std::sync::Arc;
use std::sync::atomic::AtomicU32;

#[cfg(feature = "fireframe")]
use crate::project::sandbox_manger::SandboxManager;

pub mod db_synced_crdt;
pub mod filesystem;
pub mod zip_fs;

#[cfg(feature = "fireframe")]
pub mod sandbox;
#[cfg(feature = "fireframe")]
pub mod sandbox_manger;

#[cfg(any(test, not(feature = "fireframe")))]
pub mod disk_fs;

pub struct Project {
    pub db: SqlitePool,
    pub crdt: DbSyncedCRDT,
    pub connections: AtomicU32,
    #[cfg(feature = "fireframe")]
    sandbox_manager: Arc<SandboxManager>,
}

impl Project {
    pub async fn new(
        db: SqlitePool,
        id: ProjectId,
        doc: y_octo::Doc,
        #[cfg(feature = "fireframe")]
        sandbox_manager: Arc<SandboxManager>,
    ) -> Self {
        Project {
            db: db.clone(),
            crdt: DbSyncedCRDT::new(db, id, doc),
            connections: Default::default(),
            #[cfg(feature = "fireframe")]
            sandbox_manager,
        }
    }

    #[cfg(feature = "fireframe")]
    pub async fn compile(&self, uuid: String, compiler: Compiler) -> Result<CompilationResult, String> {
        use tokio::sync::mpsc;
        use tracing::{error, info};

        let Some(initial_path) = self.crdt.lookup_uuid(&uuid) else {
            return Err(format!("No file for {uuid}"));
        };

        let id = self.crdt.id();
        let Some(script) = compiler.build_remote_script(&format!("{initial_path}")) else {
            return Err("Unable to build script".to_string());
        };
        info!("Running script: {}", &script);

        let sandbox = match self.sandbox_manager.spawn(id).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed to spawn sandbox: {e}")),
        };
        if let Err(e) = self.crdt.write_to_fs(sandbox.as_ref()).await {
            if let Err(e) = self.sandbox_manager.kill_vm(id).await {
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
        if let Err(e) = self.sandbox_manager.kill_vm(id).await {
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


    /// Returns compilation result or private error on misuse.
    #[cfg(not(feature = "fireframe"))]
    pub async fn compile(&self, uuid: String, compiler: Compiler) -> Result<CompilationResult, String> {
        use tracing::warn;
        use crate::compilation::compile_insecure;

        warn!("Running insecure compilation");
        let Some(initial_path) = self.crdt.lookup_uuid(&uuid) else {
            return Err(format!("No file for {uuid}"));
        };

        let dir = tempfile::tempdir().unwrap();
        let mut fs = disk_fs::DiskFS::new(dir.path().into());
        if let Err(e) = self.crdt.write_to_fs(&mut fs).await {
            return Err(format!("Failed to write project to disk: {e}"))
        }

        let document_path = match fs.validated_path(&initial_path) {
            Ok(v) => v,
            Err(e) => return Err(format!("Compile received for invalid path: {e}")),
        };

        let result = compile_insecure(fs.root(), &document_path, compiler).await;
        self.update_latest_compilation(&result.document).await;

        Ok(result)
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
