use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum HostCommand {
    /// Removes all files and directories in the project directory
    Clear,
    /// Creates a file relative to the project directory at [path] with contents [content]
    CreateFile { path: String, content: Vec<u8> },
    /// Creates a directory relative to the project directory at [path]
    CreateDir { path: String },
    /// Runs a sh command in the project directory
    Run (String),
    /// Triggers a [GuestMessage::FileContents] response with the contents of the passed path
    ReadFile (String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommandResult {
    Completed,
    /// Return value of a [HostCommand::Run].
    ChildExit(i32),
    Log(String),
    /// Contents of a [HostCommand::ReadFile], if that file exists
    FileContents(Option<Vec<u8>>),
}