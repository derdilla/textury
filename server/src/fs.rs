use futures::prelude::*;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::app::{FileMetadata, FileType};

#[derive(Clone)]
pub struct FS {
    root: PathBuf,
}

impl FS {
    pub fn new(root: PathBuf) -> Self {
        assert!(root.exists());
        FS {
            root: root.canonicalize().unwrap(),
        }
    }

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

    pub async fn create_file(
        &self,
        path: PathBuf,
        initial_content: Vec<u8>,
    ) -> OperationResult<()> {
        let path = self.validated_path(path)?;
        if path.exists() {
            return Err("File already exists".into());
        }
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }
        fs::write(path, initial_content)
            .map_err(|e| e.to_string())
            .await
    }

    pub async fn create_dir(&self, path: PathBuf) -> OperationResult<()> {
        let path = self.validated_path(path)?;
        if path.exists() {
            return Err("File already exists".into());
        }
        fs::create_dir_all(path).map_err(|e| e.to_string()).await
    }

    pub async fn delete_entry(&self, path: PathBuf) -> OperationResult<()> {
        let path = self.validated_path(path)?;
        if !path.exists() {
            return Err(format!("Element doesn't exist: {:?}", path));
        }
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| e.to_string()).await
        } else {
            fs::remove_file(path).map_err(|e| e.to_string()).await
        }
    }

    pub async fn read_file(&self, path: PathBuf) -> OperationResult<Vec<u8>> {
        let path = self.validated_path(path)?;
        fs::read(path).map_err(|e| e.to_string()).await
    }

    pub async fn write_file(
        &self,
        path: PathBuf,
        content: impl AsRef<[u8]>,
    ) -> OperationResult<()> {
        let path = self.validated_path(path)?;
        fs::write(path, content).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn index(&self) -> OperationResult<FileMetadata> {
        FileMetadata::serialize(&self.root, None).await
    }
}

impl FileMetadata {
    /// When [parent_path] the current directory is excluded from whole_path and the path starts at the next level.
    async fn serialize(
        path: impl AsRef<Path>,
        parent_path: Option<&str>,
    ) -> Result<FileMetadata, String> {
        let path = path.as_ref();
        let name = {
            let Some(file_name) = path.file_name() else {
                return Err("Unable to extract file name".into());
            };
            let Some(file_name) = file_name.to_str() else {
                return Err("Unable to decode file name".into());
            };
            file_name.to_string()
        };
        let whole_path = if let Some(parent_path) = parent_path {
            format!("{}/{}", parent_path, name)
        } else {
            String::new()
        };
        let file_type = {
            if !path.exists() {
                return Err("File does not exist".to_string());
            }
            let Some(metadata) = path.metadata().ok() else {
                return Err("Can't extract metadata".to_string());
            };
            if metadata.is_file() {
                FileType::Text {
                    size: metadata.len() as usize,
                }
            } else if metadata.is_dir() {
                let mut paths = fs::read_dir(path).await.map_err(|e| e.to_string())?;
                let mut children = vec![];
                while let Some(entry) = paths.next_entry().await.map_err(|e| e.to_string())? {
                    let path = entry.path().clone();
                    let next =
                        FileMetadata::serialize(path, Some(&whole_path)).map_err(|e| e.to_string());
                    children.push(next);
                }
                let children = future::join_all(children).await;
                let children: Result<Vec<FileMetadata>, String> = children.into_iter().collect();
                FileType::Directory {
                    children: children?,
                }
            } else {
                return Err("Unhandled file type".to_string());
            }
        };
        Ok(FileMetadata {
            name,
            file_type,
            whole_path,
        })
    }
}

pub type OperationResult<T> = Result<T, String>;
