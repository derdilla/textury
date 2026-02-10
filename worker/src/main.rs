use std::fs;
use std::os::unix::fs::FileTypeExt;
use std::path::PathBuf;
use worker_api::{FileCreationRequest, FileMetadata, OperationResult, TextFileUpdate, WorkerService};
use tarpc::context::Context;
use futures::{future, prelude::*};
use tarpc::server;
use tarpc::server::Channel;

#[derive(Clone)]
struct App {
    root: PathBuf,
}

impl WorkerService for App {
    async fn create_file(self, _context: Context, req: FileCreationRequest) -> OperationResult<()> {
        let path = self.root.join(req.path);
        if path.exists() {
            return Err("File already exists".into())
        }
        fs::write(path, req.initial_content.unwrap_or(vec![]))
            .map_err(|e| e.to_string())
    }

    async fn create_dir(self, _context: Context, path: PathBuf) -> OperationResult<()> {
        let path = self.root.join(path);
        if path.exists() {
            return Err("File already exists".into())
        }
        fs::create_dir_all(path)
            .map_err(|e| e.to_string())
    }

    async fn delete_entry(self, _context: Context, path: PathBuf) -> OperationResult<()> {
        let path = self.root.join(path);
        if !path.exists() {
            return Err("Element doesn't exist".into())
        }
        if path.is_dir() {
            fs::remove_dir_all(path)
                .map_err(|e| e.to_string())
        } else {
            fs::remove_file(path)
                .map_err(|e| e.to_string())
        }
    }

    async fn read_file(self, _context: Context, path: PathBuf) -> OperationResult<Vec<u8>> {
        let path = self.root.join(path);
        fs::read(path)
            .map_err(|e| e.to_string())
    }

    async fn index(self, _context: Context) -> OperationResult<FileMetadata> {
        FileMetadata::try_from(&self.root)
    }

    async fn update_file(self, _context: Context, _update: TextFileUpdate) -> OperationResult<()> {
        todo!()
    }
}


async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

#[tokio::main]
async fn main() {
    use tarpc::tokio_serde::formats::Bincode;
    use tarpc::serde_transport;

    let mut args = std::env::args().skip(1);
    let Some(socket) = args.next() else {
        println!("Usage: worker <socket> <rootDirectory>");
        return;
    };
    let socket = PathBuf::from(socket);
    /*if !socket.metadata().is_ok_and(|f| f.file_type().is_socket()) {
        println!("{} is not a socket", socket.display());
        return;
    }*/

    let Some(root) = args.next() else {
        println!("Usage: worker <rootDirectory>");
        return;
    };
    let root = PathBuf::from(root);
    if !root.is_dir() {
        println!("Root '{}' is not a directory", root.display());
        return;
    }

    let Ok(mut listener) = serde_transport::unix::listen(socket, Bincode::default).await else {
        println!("Failed to bind to socket");
        return;
    };

    // Max 200 MB messages
    listener.config_mut().max_frame_length(200 * 1024 * 1024);
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .map(|channel| {
            let app = App { root: root.clone() };
            channel.execute(app.serve()).for_each(spawn)
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;
}
