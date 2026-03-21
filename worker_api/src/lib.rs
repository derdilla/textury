use serde::{Deserialize, Serialize};
use std::fs;

#[tarpc::service]
pub trait WorkerService {
    /// Create a new file within the project root or updates it.
    async fn create_file(req: FileCreationRequest) -> OperationResult<()>;

    /// Create a new directory within the project root.
    async fn create_dir(path: std::path::PathBuf) -> OperationResult<()>;

    /// Delete a file or directory within the project root.
    async fn delete_entry(path: std::path::PathBuf) -> OperationResult<()>;

    // TODO: this sends no matter the size, might add a warning or pre-open check to the client.
    // Do something in the implementation to prevent resource exhaustion.
    async fn read_file(path: std::path::PathBuf) -> OperationResult<Vec<u8>>;

    /// The contents of the working directory.
    async fn index() -> OperationResult<FileMetadata>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCreationRequest {
    pub path: std::path::PathBuf,
    pub initial_content: Option<Vec<u8>>,
}

pub type OperationResult<T> = Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileMetadata {
    pub name: String,
    pub file_type: FileType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    Text { size: usize },
    Binary { size: usize },
    Directory { children: Vec<FileMetadata> },
}

impl TryFrom<&std::path::PathBuf> for FileMetadata {
    type Error = String;

    fn try_from(value: &std::path::PathBuf) -> Result<FileMetadata, Self::Error> {
        let file_type = {
            if !value.exists() {
                return Err("File does not exist".to_string());
            }
            let Some(metadata) = value.metadata().ok() else {
                return Err("Can't extract metadata".to_string());
            };
            if metadata.is_file() {
                FileType::Text {
                    size: metadata.len() as usize,
                }
            } else if metadata.is_dir() {
                let children = fs::read_dir(value)
                    .map_err(|e| e.to_string())?
                    .map(|e| {
                        let path = e.map_err(|err| err.to_string())?.path();
                        FileMetadata::try_from(&path).map_err(|e| e.to_string())
                    })
                    .collect::<Result<Vec<FileMetadata>, String>>()?;
                FileType::Directory { children }
            } else {
                return Err("Unhandled file type".to_string());
            }
        };
        let name = {
            let Some(file_name) = value.file_name() else {
                return Err("Unable to extract file name".into());
            };
            let Some(file_name) = file_name.to_str() else {
                return Err("Unable to decode file name".into());
            };
            file_name.to_string()
        };
        Ok(FileMetadata { name, file_type })
    }
}
