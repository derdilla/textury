use futures::prelude::*;
use std::path::{Path, PathBuf};
use tokio::{fs};

#[cfg(test)]
use serde::{Deserialize, Serialize};
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

    #[cfg(test)]
    pub async fn index(&self) -> OperationResult<FileMetadata> {
        FileMetadata::serialize(&self.root, None).await
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

#[cfg(test)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg(test)]
pub struct FileMetadata {
    pub whole_path: String,
    pub name: String,
    pub file_type: FileType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg(test)]
pub enum FileType {
    Text { size: usize },
    Binary { size: usize },
    Directory { children: Vec<FileMetadata> },
}

pub type OperationResult<T> = Result<T, String>;
