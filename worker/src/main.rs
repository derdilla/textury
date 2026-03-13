mod types;

use axum::extract::{Path, WebSocketUpgrade};
use axum::response::{Response, Result};
use axum::{Json, extract::State};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tempfile::tempfile;
use tokio::sync::RwLock;
use yrs_tokio::broadcast::BroadcastGroup;

use crate::types::{FileCreationRequest, FileMetadata};

#[derive(Clone)]
struct App {
    root: PathBuf,
    // TODO: sync changes with fs
    open_documents: Arc<RwLock<HashMap<String, Arc<BroadcastGroup>>>>,
}

type OperationResult<T> = Result<T, String>;

async fn create_file(
    State(app): State<App>,
    Json(req): Json<FileCreationRequest>,
) -> OperationResult<()> {
    let path = app.root.join(req.path);
    if path.exists() {
        return Err("File already exists".into());
    }
    fs::write(path, req.initial_content.unwrap_or(vec![])).map_err(|e| e.to_string())
}

async fn create_dir(State(app): State<App>, Json(path): Json<PathBuf>) -> OperationResult<()> {
    let path = app.root.join(path);
    if path.exists() {
        return Err("File already exists".into());
    }
    fs::create_dir_all(path).map_err(|e| e.to_string())
}

async fn delete_file(State(app): State<App>, Json(path): Json<PathBuf>) -> OperationResult<()> {
    let path = app.root.join(path);
    if !path.exists() {
        return Err("Element doesn't exist".into());
    }
    if path.is_dir() {
        return Err("Not a file".into());
    } else {
        fs::remove_file(path).map_err(|e| e.to_string())
    }
}

async fn delete_dir(State(app): State<App>, Json(path): Json<PathBuf>) -> OperationResult<()> {
    let path = app.root.join(path);
    if !path.exists() {
        return Err("Element doesn't exist".into());
    }
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|e| e.to_string())
    } else {
        return Err("Not a directory".into());
    }
}

async fn read_file(State(app): State<App>, Json(path): Json<PathBuf>) -> OperationResult<Vec<u8>> {
    let path = app.root.join(path);
    fs::read(path).map_err(|e| e.to_string())
}

async fn index(State(app): State<App>) -> OperationResult<Json<FileMetadata>> {
    FileMetadata::try_from(&app.root).map(|e| Json(e))
}

async fn export(State(app): State<App>) -> OperationResult<Vec<u8>> {
    let mut tar_file = tempfile().unwrap();
    tar::Builder::new(&tar_file)
        .append_dir(".", &app.root)
        .map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    tar_file
        .read_to_end(&mut result)
        .map_err(|_| String::from("Can't read file"))?;
    Ok(result)
}

pub(crate) async fn edit(
    ws: WebSocketUpgrade,
    State(app): State<App>,
    Path(file_path): Path<String>,
) -> Response {
    ws.on_upgrade(|socket| editor::open_document(app, socket, file_path))
}

mod editor {
    use std::sync::Arc;

    use axum::extract::ws::{Message, WebSocket};
    use futures::StreamExt;
    use futures_util::Sink;
    use futures_util::stream::{SplitSink, SplitStream};
    use tokio::sync::{Mutex, RwLock};
    use yrs::Doc;
    use yrs_tokio::broadcast::BroadcastGroup;
    use yrs_tokio::signaling::Message as SignalingMessage;
    /*use yrs_tokio::{
        YrsExchange, YrsSink, YrsStream, impl_yrs_signal_stream, to_signaling_message,
        yrs_common_sink,
    };*/
    use yrs_tokio::{YrsSink, YrsStream, yrs_common_sink};

    use crate::App;

    #[derive(YrsStream)]
    pub struct YrsStream(SplitStream<WebSocket>);

    //#[derive(YrsExchange)]
    //pub struct YrsSignalStream(SplitStream<WebSocket>);
    //impl_yrs_signal_stream!(YrsSignalStream, item => to_signaling_message!(item));

    #[derive(YrsSink)]
    pub struct YrsSink(SplitSink<WebSocket, Message>);
    #[yrs_common_sink]
    impl Sink<SignalingMessage> for YrsSink {}

    pub async fn open_document(app: App, ws: WebSocket, file_path: String) {
        let mut open_documents = app.open_documents.write().await;
        if !open_documents.contains_key(&file_path) {
            let doc = Doc::new();
            {
                use yrs::{Text, Transact};

                let txt = doc.get_or_insert_text("codemirror");
                let mut txn = doc.transact_mut();
                txt.push(&mut txn, "Initial text");
            }
            let awareness = yrs::sync::awareness::Awareness::new(doc);
            let awareness = Arc::new(RwLock::new(awareness));
            let bcast = Arc::new(BroadcastGroup::new(awareness.clone(), 32).await);
            open_documents.insert(file_path.clone(), bcast);
        }

        let bcast: Arc<BroadcastGroup> = open_documents.get(&file_path).unwrap().clone();
        drop(open_documents);

        let (sink, stream) = ws.split();
        let sink = Arc::new(Mutex::new(YrsSink::from(sink)));
        let stream = YrsStream::from(stream);
        let sub = bcast.subscribe::<YrsSink, YrsStream, _>(sink, stream);
        if let Err(_) = sub.completed().await {
            //warn!("Error: {}", e);
        }
    }
}

#[tokio::main]
async fn main() {
    use axum::Router;
    use axum::routing::{any, get, post};

    let app = App {
        root: PathBuf::from_str("/project").unwrap(),
        open_documents: Arc::new(RwLock::new(HashMap::new())),
    };

    // build our application with a route
    let app = Router::new()
        .route("/", get(index))
        .route(
            "/file",
            get(read_file).post(create_file).delete(delete_file),
        )
        .route("/dir", post(create_dir).delete(delete_dir))
        .route("/export", get(export))
        .route("/edit/{path}", any(edit))
        .with_state(app);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
