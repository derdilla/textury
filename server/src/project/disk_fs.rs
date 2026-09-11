use futures::prelude::*;
use std::path::{Path, PathBuf};
use tokio::{fs};

use crate::project::filesystem::Filesystem;

#[derive(Clone)]
pub struct DiskFS {
    root: PathBuf,
}

impl DiskFS {
    pub fn new(root: PathBuf) -> Self {
        if !root.exists() {
            std::fs::create_dir_all(&root).expect("Unable to create new project");
        }
        DiskFS {
            root: root.canonicalize().unwrap(),
        }
    }

    #[cfg(not(feature = "fireframe"))]
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Determine absolute fs path of [path], if we are certain it is within [root].
    pub fn validated_path(&self, path: impl AsRef<Path>) -> OperationResult<PathBuf> {
        // When we get a non-relative path, we assume it is relative
        let path = match path.as_ref().strip_prefix("/") {
            Ok(path) => path,
            Err(_) => path.as_ref(),
        };
        if path.starts_with("/") {
            return Err("Path shouldn't start with multiple leading /".into());
        }

        let path = self.root.join(path);

        // If we can resolve the path permission checking becomes easy
        if path.exists() {
            let canonical = path.canonicalize().map_err(|e| e.to_string())?;
            return if canonical.starts_with(&self.root) {
                Ok(canonical)
            } else {
                Err("Path outside root!".into())
            };
        }

        // Conservatively prohibit all relative paths
        if path.starts_with(&self.root) && path.to_str().is_some_and(|p| !p.contains("..")) {
            // TODO: we could validate no component is a symlink,
            //       but what if an attacker causes some sort of race condition?
            return Ok(path);
        }
        Err("Unable to determine path is safe".into())
    }
}

impl Filesystem for DiskFS {
    async fn create_file(&self, path: String, content: &Vec<u8>) -> Result<(), String> {
        let path = self.validated_path(path)?;
        if path.exists() {
            return Err(format!("File already exists: {}", path.display()));
        }
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }
        fs::write(path, content)
            .map_err(|e| e.to_string())
            .await
    }

    async fn create_dir(&self, path: String) -> Result<(), String> {
        let path = self.validated_path(path)?;
        if path.exists() {
            return Err(format!("File already exists: {}", path.display()));
        }
        fs::create_dir_all(path).map_err(|e| e.to_string()).await
    }
}

pub type OperationResult<T> = Result<T, String>;
