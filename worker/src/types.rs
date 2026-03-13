use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCreationRequest {
    pub path: std::path::PathBuf,
    pub initial_content: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub name: String,
    pub file_type: FileType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
                let children = std::fs::read_dir(value)
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
