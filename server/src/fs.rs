use async_zip::{Compression, ZipEntryBuilder, base::write::ZipFileWriter};
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::{fs, io::DuplexStream};
use tracing::error;
use walkdir::WalkDir;

#[derive(Clone)]
pub struct FS {
    root: PathBuf,
}

impl FS {
    pub fn new(root: PathBuf) -> Self {
        if !root.exists() {
            std::fs::create_dir_all(&root).expect("Unable to create new project");
        }
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
            return Err(format!("File already exists: {}", path.display()));
        }
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }
        fs::write(path, initial_content)
            .map_err(|e| e.to_string())
            .await
    }

    pub async fn create_dir(&self, path: PathBuf) -> OperationResult<()> {
        let path = self.validated_path(path)?;
        if path.exists() {
            return Err(format!("File already exists: {}", path.display()));
        }
        fs::create_dir_all(path).map_err(|e| e.to_string()).await
    }

    pub async fn index(&self) -> OperationResult<FileMetadata> {
        FileMetadata::serialize(&self.root, None).await
    }

    pub fn zip(&self) -> DuplexStream {
        let (reader, writer) = tokio::io::duplex(4096);
        let root = self.root.clone();
        tokio::spawn(async move {
            let walkdir = WalkDir::new(&root);
            let mut zip = ZipFileWriter::with_tokio(writer);

            for entry_result in walkdir.into_iter() {
                let entry = match entry_result {
                    Ok(entry) => entry,
                    Err(e) => {
                        error!("Error while traversing directory {:?}: {e}", root);
                        return;
                    }
                };
                let path = entry.path();

                // If some unzip tools unzip files with directory paths incorrectly, we should
                // write directories explicitly.
                if !path.is_file() {
                    continue;
                }

                let path_stripped = path.strip_prefix(&root).map_err(|e| e.to_string()).unwrap();
                let Some(path_as_string) = path_stripped.to_str().map(str::to_owned) else {
                    error!("{:?} is a Non UTF-8 Path", path_stripped.display());
                    continue;
                };

                let Ok(content) = tokio::fs::read(path)
                    .await
                    .inspect_err(|e| error!("Failed reading {:?}: {e}", path_stripped.display()))
                else {
                    continue;
                };

                let builder = ZipEntryBuilder::new(path_as_string.into(), Compression::Zstd);
                if let Err(e) = zip.write_entry_whole(builder, &content).await {
                    error!(
                        "Failed writing file {:?} to zip: {e}",
                        path_stripped.display()
                    )
                }
            }

            _ = zip
                .close()
                .await
                .inspect_err(|e| error!("Failed closing zip: {e}"));
        });

        reader
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileMetadata {
    pub whole_path: String,
    pub name: String,
    pub file_type: FileType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    Text { size: usize },
    Binary { size: usize },
    Directory { children: Vec<FileMetadata> },
}

pub type OperationResult<T> = Result<T, String>;
