use tokio::sync::{mpsc};
use crate::project::filesystem::Filesystem;

pub trait Sandbox: Filesystem {
    /// Run a shell command in the sandboxed filesystem.
    /// Returns the exit code of the command, if known.
    fn run(&self, script: String, logger_tx: Option<mpsc::Sender<String>>) -> impl Future<Output = Option<i32>> + Send;

    fn read_file(&self, path: String) -> impl Future<Output = Option<Vec<u8>>> + Send;
}
