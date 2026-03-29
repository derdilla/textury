use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::app::yrs_tokio_types::{YrsSink, YrsStream};
use crate::model;
use axum::extract::ws::WebSocket;
use futures_util::StreamExt;
use rand::distr::SampleString;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, ModelTrait, SelectExt,
};
use sea_orm::{ColumnTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::ext::NumericalDuration;
use tokio::fs;
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, trace, warn};
use yrs::{Doc, GetString, Transact};
use yrs_tokio::broadcast::BroadcastGroup;

#[derive(Clone)]
pub struct App {
    conn: DatabaseConnection,
    /// Map from document ids (project_id, file_path) to open document.
    open_documents: Arc<RwLock<HashMap<(String, String), Arc<BroadcastGroup>>>>,
    /// Map from share_id (long random) to user_id
    shares: Arc<RwLock<HashMap<String, String>>>,
}

impl App {
    pub async fn new(conn: DatabaseConnection) -> Self {
        let obj = Self {
            conn,
            open_documents: Default::default(),
            shares: Default::default(),
        };
        obj.setup_schema().await;
        obj
    }

    async fn setup_schema(&self) {
        let res = self
            .conn
            .get_schema_builder()
            .register(model::user::Entity)
            .register(model::project::Entity)
            .register(model::project_memberships::Entity)
            .register(model::registration_tokens::Entity)
            .register(model::file::Entity)
            .apply(&self.conn)
            .await;
        if let Err(res) = res {
            warn!("Failed setting up db tables: {}", res)
        }
    }

    pub async fn add_registration_token(&self) -> String {
        let token = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 20);
        let expires = OffsetDateTime::now_utc();
        let expires = expires.checked_add(15.minutes()).unwrap();
        model::registration_tokens::ActiveModel {
            token: Set(token.to_string()),
            expires: Set(expires),
        }
        .insert(&self.conn)
        .await
        .unwrap();

        token.to_string()
    }

    pub async fn register(
        &self,
        registration_token: String,
        name: String,
        email: String,
        password: String,
    ) -> CreateAccountResult {
        use argon2::{Argon2, PasswordHasher, password_hash::SaltString};

        if name.len() < 4 || password.len() < 8 {
            return CreateAccountResult::BadRequest;
        }

        // Check for conflicts
        if model::user::Entity::find_by_email(&email)
            .exists(&self.conn)
            .await
            .unwrap()
        {
            return CreateAccountResult::Exists;
        }

        // Validate registration token
        let Ok(Some(token)) =
            model::registration_tokens::Entity::find_by_token(&registration_token)
                .one(&self.conn)
                .await
        else {
            return CreateAccountResult::InvalidToken;
        };
        if token.expires < OffsetDateTime::now_utc() {
            token.delete(&self.conn).await.unwrap();
            return CreateAccountResult::ExpiredToken;
        }
        token.delete(&self.conn).await.unwrap();

        // Save account
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        let argon2 = Argon2::default();
        let Ok(password_hash) = argon2.hash_password(password.as_bytes(), &salt) else {
            return CreateAccountResult::BadRequest;
        };
        let new_user = model::user::ActiveModel {
            name: Set(name),
            email: Set(email),
            password_hash: Set(password_hash.to_string()),
            ..Default::default()
        };
        let new_user = new_user.insert(&self.conn).await.unwrap();

        CreateAccountResult::Created { id: new_user.id }
    }

    /// Returns the user or none if the user doesn't exist or the password is wrong
    pub async fn login(&self, email: String, password: String) -> Option<model::user::Model> {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};

        let user_details = model::user::Entity::find_by_email(&email)
            .one(&self.conn)
            .await
            .unwrap()?;
        let parsed_hash = PasswordHash::new(&user_details.password_hash).ok()?;

        let argon2 = Argon2::default();
        argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .ok()?;

        Some(user_details)
    }

    pub async fn user_from_id(&self, user_id: i32) -> Option<model::user::Model> {
        let user = model::user::Entity::find_by_id(user_id)
            .one(&self.conn)
            .await;
        println!("User ID: {:?}", user);

        let Ok(user) = user else {
            return None;
        };

        user
    }

    pub async fn project_exists(&self, project_id: &str) -> bool {
        model::project::Entity::find_by_id(project_id)
            .exists(&self.conn)
            .await
            .unwrap()
    }

    pub async fn insert_project(&self, project_id: String, user_id: i32, title: String) -> bool {
        use sea_orm::Set;

        let new_project = model::project::ActiveModel {
            id: Set(project_id.clone()),
            owner_id: Set(user_id),
            title: Set(title),
        };
        if !new_project.insert(&self.conn).await.is_ok() {
            return false;
        }

        model::project_memberships::ActiveModel {
            user_id: Set(user_id),
            project_id: Set(project_id),
        }
        .insert(&self.conn)
        .await
        .is_ok()
    }

    /// Checks whether [user] owns project and returns whether a project was deleted
    pub async fn delete_project(
        &self,
        project_id: &str,
        user: &model::user::Model,
    ) -> Option<bool> {
        let project = model::project::Entity::find_by_id(project_id)
            .one(&self.conn)
            .await
            .ok()??;
        if project.owner_id != user.id {
            warn!(
                "Unauthorized user {} tried to delete {}",
                user.email, project.id
            );
            return None;
        }

        // TODO: close anything open
        match project.cascade_delete(&self.conn).await {
            Ok(res) => Some(res.rows_affected >= 1),
            Err(e) => {
                error!("Failed deleting: {}", e);
                None
            }
        }
    }

    pub async fn list_projects(
        &self,
        from: &model::user::Model,
    ) -> Option<Vec<model::project::Model>> {
        from.find_related(model::project::Entity)
            .all(&self.conn)
            .await
            .ok()
    }

    pub async fn project_exists_for_user(
        &self,
        project_id: &str,
        user: &model::user::Model,
    ) -> bool {
        user.find_related(model::project::Entity)
            .filter(model::project::Column::Id.eq(project_id))
            .exists(&self.conn)
            .await
            .unwrap()
    }

    pub async fn get_project_for_user(
        &self,
        project_id: &str,
        user: &model::user::Model,
    ) -> Option<model::project::Model> {
        user.find_related(model::project::Entity)
            .filter(model::project::Column::Id.eq(project_id))
            .one(&self.conn)
            .await
            .ok()?
    }

    /// Creates a share token for the project, if sharer has access
    pub async fn share(&self, project_id: &str, sharer: &model::user::Model) -> Option<String> {
        // TODO: check for actual ownership
        let Some(proj) = self.get_project_for_user(&project_id, &sharer).await else {
            return None;
        };

        if proj.owner_id == sharer.id {
            warn!(
                "User {} tried to share {} without permissions",
                sharer.name, proj.id
            );
            return None;
        }

        let key = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 40);
        let res = self
            .shares
            .write()
            .await
            .insert(key.clone(), project_id.to_string());
        debug_assert_eq!(res, None);
        Some(key)
    }

    pub async fn accept_share(&self, share_id: &str, target: &model::user::Model) -> bool {
        let project_id = {
            let mut shares = self.shares.write().await;
            let Some(project_id) = shares.remove(share_id) else {
                info!("Attempted to accept non-existent share: {}", share_id);
                return false;
            };
            project_id
        };

        if self.project_exists_for_user(&project_id, &target).await {
            return true;
        }

        if !self.project_exists(&project_id).await {
            // Project has been deleted since sharing
            return false;
        }

        model::project_memberships::ActiveModel {
            user_id: Set(target.id),
            project_id: Set(project_id.to_string()),
        }
        .insert(&self.conn)
        .await
        .is_ok()
    }

    pub async fn list_files(&self, project_id: &str) -> Option<FileMetadata> {
        let file_list = model::file::Entity::find()
            .filter(model::file::Column::ProjectId.eq(project_id))
            .all(&self.conn)
            .await
            .unwrap();

        let mut result = FileMetadata {
            name: String::new(),
            file_type: FileType::Directory {
                children: Vec::new(),
            },
            whole_path: String::new(),
        };
        for file in file_list {
            if !insert_file(&mut result, file, String::new()) {
                warn!("Failed to insert a file into response");
                return None;
            }
        }

        Some(result)
    }

    pub async fn add_or_update_file(
        &self,
        project_id: String,
        full_file_path: String,
        content: Vec<u8>,
    ) -> bool {
        let existing_check =
            model::file::Entity::find_by_id((project_id.clone(), full_file_path.clone()))
                .exists(&self.conn);
        let new_entry = model::file::ActiveModel {
            project_id: Set(project_id),
            path: Set(full_file_path),
            content: Set(content),
        };
        if existing_check.await.is_ok_and(|e| e) {
            new_entry.save(&self.conn).await.is_ok()
        } else {
            new_entry.insert(&self.conn).await.is_ok()
        }
    }

    pub async fn delete_file(&self, project_id: String, full_file_path: String) -> bool {
        model::file::Entity::delete_by_id((project_id, full_file_path))
            .exec(&self.conn)
            .await
            .is_ok()
    }

    pub async fn get_file(&self, project_id: String, full_file_path: String) -> Option<Vec<u8>> {
        model::file::Entity::find_by_id((project_id, full_file_path))
            .one(&self.conn)
            .await
            .unwrap()
            .map(|e| e.content)
    }

    /// Handles websocket as reference to document_id (project_id, file_path).
    pub async fn open_document(&self, ws: WebSocket, document_id: (String, String)) {
        let mut open_documents = self.open_documents.write().await;

        // Open document if currently closed
        if !open_documents.contains_key(&document_id) {
            use yrs::{Text, Transact};

            info!("Creating new Document");
            let (project_id, document_path) = document_id.clone();
            let doc = Doc::new();

            // Load initial text from db
            let current_text = model::file::Entity::find_by_id(document_id.clone())
                .one(&self.conn)
                .await;
            if let Ok(Some(content)) = current_text {
                if let Ok(content) = str::from_utf8(&content.content) {
                    let txt = doc.get_or_insert_text("codemirror");
                    let mut txn = doc.transact_mut();
                    txt.push(&mut txn, content);
                }
            }

            // Insert into open docuemnts
            let awareness = yrs::sync::awareness::Awareness::new(doc);
            let awareness = Arc::new(RwLock::new(awareness));
            let bcast = Arc::new(BroadcastGroup::new(awareness.clone(), 32).await);
            open_documents.insert(document_id.clone(), bcast);

            // Create autosave timer
            let app = self.clone();
            tokio::task::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(5));
                loop {
                    interval.tick().await;
                    if !app.save(project_id.clone(), document_path.clone()).await {
                        info!(
                            "Stopping autosave timer for {} in {}",
                            document_path, project_id
                        );
                        return;
                    }
                }
            });
        }

        // Get open document
        let bcast: Arc<BroadcastGroup> = open_documents.get(&document_id).unwrap().clone();
        drop(open_documents);

        // Create subscription for client
        let (sink, stream) = ws.split();
        let sink = Arc::new(Mutex::new(YrsSink::from(sink)));
        let stream = YrsStream::from(stream);
        let sub = bcast.subscribe::<YrsSink, YrsStream, _>(sink, stream);

        // Save document once client disconnects
        if let Err(e) = sub.completed().await {
            warn!("Error: {}", e);
        }
        let (project_id, document_path) = document_id;
        self.save(project_id, document_path).await;
    }

    pub async fn save(&self, project_id: String, document_path: String) -> bool {
        let doc = self.open_documents.read().await;
        let Some(doc) = doc.get(&(project_id.clone(), document_path.clone())) else {
            warn!("Unable to aquire '{}' in {}", document_path, project_id);
            return false;
        };
        let doc = doc.awareness().read().await;
        let doc = doc.doc();
        let text = doc.get_or_insert_text("codemirror");
        let text = text.get_string(&doc.transact());

        // TODO: create file in db (/reject) if necessary
        model::file::ActiveModel {
            project_id: Set(project_id),
            path: Set(document_path),
            content: Set(text.into_bytes()),
        }
        .update(&self.conn)
        .await
        .is_ok()
    }

    pub async fn compile(
        &self,
        project_id: String,
        document_path: String,
        compiler: Option<Compiler>,
    ) -> CompilationResult {
        use base64::prelude::*;
        use std::process::Command;

        if !self.save(project_id.clone(), document_path.clone()).await {
            return CompilationResult::err("Couldn't save open document".into());
        }

        // TODO: containerize
        let dir = tempfile::tempdir().unwrap();
        let files = model::file::Entity::find()
            .filter(model::file::Column::ProjectId.eq(project_id))
            .all(&self.conn)
            .await;
        let files = match files {
            Ok(files) => files,
            Err(err) => {
                warn!("Failed to read project documents from db: {}", err);
                return CompilationResult::err("Failed to read project documents from db".into());
            }
        };
        for f in files {
            let path = dir.path().join(f.path);
            trace!("Writing: '{}'", &path.display());
            if let Some(partent_dir) = path.parent() {
                if let Err(e) = fs::create_dir_all(partent_dir).await {
                    warn!("Failed to write directory {}", e);
                    return CompilationResult::err("Failed to write project to disk".into());
                }
            }
            if let Err(e) = fs::write(path, f.content).await {
                warn!("Failed to write file {}", e);
                return CompilationResult::err("Failed to write project to disk".into());
            };
        }

        let output_path = dir.path().join("dilladocs_output.pdf");

        let compiler = compiler.unwrap_or_else(|| {
            if document_path.ends_with("typ") {
                return Compiler::Typst;
            } else if document_path.ends_with("tex") {
                return Compiler::PdfLatex;
            }
            warn!("Couldn't determine file format");
            return Compiler::PdfLatex;
        });

        let log = match compiler {
            Compiler::PdfLatex => Command::new("pdflatex")
                .current_dir(&dir)
                .args([
                    &format!(
                        "-output-directory={}",
                        dir.path().as_os_str().to_str().unwrap()
                    ),
                    "-jobname=dilladocs_output",
                    "-output-format=pdf",
                    &document_path,
                ])
                .output()
                .map(|output| String::from_utf8(output.stdout).unwrap())
                .unwrap_or("Output capture failed".into()),
            Compiler::XeLatex => "XeLatex compiler unimplemented".into(),
            Compiler::LuaLatex => "LuaLatex compiler unimplemented".into(),
            Compiler::Typst => {
                // typst compile foo/main.typ output.pdf
                Command::new("typst")
                    .current_dir(&dir)
                    .args([
                        "compile",
                        &document_path,
                        &output_path.as_os_str().to_str().unwrap(),
                    ])
                    .output()
                    .map(|output| String::from_utf8(output.stdout).unwrap())
                    .unwrap_or("Output capture failed".into())
            }
        };

        let output: Result<Vec<u8>, _> = fs::read(output_path).await;
        let output = output.map(|data| {
            format!(
                "data:application/pdf;base64,{}",
                BASE64_STANDARD.encode(data)
            )
        });

        CompilationResult {
            log,
            document: output.ok(),
        }
    }
}

pub fn insert_file(
    root: &mut FileMetadata,
    file: model::file::Model,
    whole_path_prefix: String,
) -> bool {
    let mut path = file.path;
    while path.starts_with("/") {
        log::warn!("Inserted file started with '/'. Interpreting as relative to root.");
        path.remove(0);
    }
    let FileType::Directory { children } = &mut root.file_type else {
        return false;
    };
    if let Some((name, path)) = path.split_once("/") {
        // is directory
        let whole_path = format!("{}/{}", whole_path_prefix, &name);
        if children.iter().find(|e| &e.name == &name).is_none() {
            let root = FileMetadata {
                whole_path: whole_path.clone(),
                name: name.to_string(),
                file_type: FileType::Directory {
                    children: Vec::new(),
                },
            };
            children.push(root);
        }
        let new_root = children.iter_mut().find(|e| e.name == name).unwrap();
        insert_file(
            new_root,
            model::file::Model {
                project_id: file.project_id,
                path: path.to_string(),
                content: file.content,
            },
            whole_path,
        )
    } else {
        // is the lasst file
        let whole_path = format!("{}/{}", whole_path_prefix, &path);
        children.push(FileMetadata {
            name: path,
            file_type: FileType::Binary {
                // TODO: specify
                size: file.content.len(),
            },
            whole_path,
        });
        true
    }
}

#[derive(Debug, PartialEq)]
pub enum CreateAccountResult {
    Created {
        id: i32,
    },
    Exists,
    InvalidToken,
    ExpiredToken,
    /// Name too short, password too weak
    BadRequest,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompilationResult {
    pub log: String,
    /// data URL of the document.
    pub document: Option<String>,
}

impl CompilationResult {
    pub fn err(error: String) -> Self {
        CompilationResult {
            log: error,
            document: None,
        }
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

#[derive(Serialize, Deserialize, Debug)]
pub enum Compiler {
    PdfLatex,
    XeLatex,
    LuaLatex,
    Typst,
}

mod yrs_tokio_types {
    use axum::extract::ws::{Message, WebSocket};
    use futures_util::Sink;
    use futures_util::stream::{SplitSink, SplitStream};
    use yrs_tokio::signaling::Message as SignalingMessage;
    use yrs_tokio::{
        YrsExchange, YrsSink, YrsStream, impl_yrs_signal_stream, to_signaling_message,
        yrs_common_sink,
    };

    #[derive(YrsStream)]
    pub struct YrsStream(SplitStream<WebSocket>);

    #[allow(dead_code)]
    #[derive(YrsExchange)]
    pub struct YrsSignalStream(SplitStream<WebSocket>);

    impl_yrs_signal_stream!(YrsSignalStream, item => to_signaling_message!(item));

    #[derive(YrsSink)]
    pub struct YrsSink(SplitSink<WebSocket, Message>);
    #[yrs_common_sink]
    impl Sink<SignalingMessage> for YrsSink {}
}

#[cfg(test)]
mod tests {
    use sea_orm::Database;

    use crate::app::{App, CreateAccountResult, FileMetadata, FileType};

    #[tokio::test]
    async fn can_create_accounts() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

        let token = app.add_registration_token().await;
        let res = app
            .register(
                token,
                "foobar".into(),
                "foo@bar.de".into(),
                "foobar!1sadf".into(),
            )
            .await;
        assert!(
            match res {
                CreateAccountResult::Created { .. } => true,
                _ => false,
            },
            "{:?}",
            res,
        );
    }

    #[tokio::test]
    async fn can_login() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

        let token = app.add_registration_token().await;
        let res = app
            .register(
                token,
                "foobar".into(),
                "foo@bar.de".into(),
                "foobar!1sadf".into(),
            )
            .await;
        assert!(
            match res {
                CreateAccountResult::Created { .. } => true,
                _ => false,
            },
            "{:?}",
            res,
        );

        let res = app.login("foo@bar.de".into(), "foobar!1sadf".into()).await;
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name, "foobar");
        assert_eq!(res.email, "foo@bar.de");
    }

    #[tokio::test]
    async fn cant_create_accounts_without_token() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

        let res = app
            .register(
                "not a token".into(),
                "foobar".into(),
                "foo@bar.de".into(),
                "foobar!1sadf".into(),
            )
            .await;
        assert_eq!(res, CreateAccountResult::InvalidToken);
    }

    #[tokio::test]
    async fn doesnt_generate_same_token_twice() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

        let token1 = app.add_registration_token().await;
        let token2 = app.add_registration_token().await;
        assert!(token1 != token2);
    }

    #[tokio::test]
    async fn cant_login_with_wrong_password() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

        let token = app.add_registration_token().await;
        let res = app
            .register(
                token,
                "foobar".into(),
                "foo@bar.de".into(),
                "foobar!1sadf".into(),
            )
            .await;
        assert!(
            match res {
                CreateAccountResult::Created { .. } => true,
                _ => false,
            },
            "{:?}",
            res,
        );

        let res = app.login("foo@bar.de".into(), "wrong pw".into()).await;
        assert!(res.is_none());
    }

    #[tokio::test]
    async fn can_create_and_list_projects() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let app = App::new(db).await;

        let token = app.add_registration_token().await;
        let res = app
            .register(
                token,
                "foobar".into(),
                "foo@bar.de".into(),
                "foobar!1sadf".into(),
            )
            .await;
        let user_id = match res {
            CreateAccountResult::Created { id } => id,
            _ => panic!("Failed to create account"),
        };
        let from = app.user_from_id(user_id).await.unwrap();

        let projects = app.list_projects(&from).await;
        assert!(projects.is_some_and(|projects| projects.is_empty()));

        let res = app
            .insert_project("test_project".into(), user_id, "Test Title".into())
            .await;
        assert!(res, "Project creation failed");

        let projects = app.list_projects(&from).await.unwrap();
        assert_eq!(projects.len(), 1);

        let res = app
            .insert_project("test_project2".into(), user_id, "Test Title".into())
            .await;
        assert!(res, "Project creation failed");

        let projects = app.list_projects(&from).await.unwrap();
        dbg!(&projects);
        assert_eq!(projects.len(), 2);

        let res = app
            .insert_project("test_project".into(), user_id, "Test Title".into())
            .await;
        assert!(!res, "Created duplicate projects");

        let projects = app.list_projects(&from).await.unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn converts_file_list_to_hierarchie() {
        use super::insert_file;
        use crate::model;

        let mut root = FileMetadata {
            name: String::new(),
            whole_path: String::new(),
            file_type: FileType::Directory {
                children: Vec::new(),
            },
        };
        let file1 = model::file::Model {
            project_id: String::new(),
            path: "this/was/a/triump!".to_string(),
            content: Vec::new(),
        };
        let result = insert_file(&mut root, file1, String::new());
        assert!(result);
        assert_eq!(
            root.clone(),
            FileMetadata {
                name: String::new(),
                whole_path: String::new(),
                file_type: FileType::Directory {
                    children: vec![FileMetadata {
                        whole_path: "/this".into(),
                        name: "this".to_string(),
                        file_type: FileType::Directory {
                            children: vec![FileMetadata {
                                whole_path: "/this/was".into(),
                                name: "was".to_string(),
                                file_type: FileType::Directory {
                                    children: vec![FileMetadata {
                                        whole_path: "/this/was/a".into(),
                                        name: "a".to_string(),
                                        file_type: FileType::Directory {
                                            children: vec![FileMetadata {
                                                whole_path: "/this/was/a/triump!".into(),
                                                name: "triump!".to_string(),
                                                file_type: FileType::Binary { size: 0 },
                                            },],
                                        },
                                    },],
                                },
                            },],
                        },
                    },],
                },
            }
        );

        let file2 = model::file::Model {
            project_id: String::new(),
            path: "this/is/a/second file".to_string(),
            content: vec![1, 2],
        };
        let result = insert_file(&mut root, file2, String::new());
        assert!(result);
        //dbg!(root.clone());
        assert_eq!(
            root,
            FileMetadata {
                whole_path: String::new(),
                name: String::new(),
                file_type: FileType::Directory {
                    children: vec![FileMetadata {
                        whole_path: "/this".to_string(),
                        name: "this".to_string(),
                        file_type: FileType::Directory {
                            children: vec![
                                FileMetadata {
                                    whole_path: "/this/was".to_string(),
                                    name: "was".to_string(),
                                    file_type: FileType::Directory {
                                        children: vec![FileMetadata {
                                            whole_path: "/this/was/a".to_string(),
                                            name: "a".to_string(),
                                            file_type: FileType::Directory {
                                                children: vec![FileMetadata {
                                                    whole_path: "/this/was/a/triump!".to_string(),
                                                    name: "triump!".to_string(),
                                                    file_type: FileType::Binary { size: 0 },
                                                },],
                                            },
                                        },],
                                    },
                                },
                                FileMetadata {
                                    whole_path: "/this/is".to_string(),
                                    name: "is".to_string(),
                                    file_type: FileType::Directory {
                                        children: vec![FileMetadata {
                                            whole_path: "/this/is/a".to_string(),
                                            name: "a".to_string(),
                                            file_type: FileType::Directory {
                                                children: vec![FileMetadata {
                                                    whole_path: "/this/is/a/second file"
                                                        .to_string(),
                                                    name: "second file".to_string(),
                                                    file_type: FileType::Binary { size: 2 },
                                                },],
                                            },
                                        },],
                                    },
                                },
                            ],
                        },
                    },],
                },
            }
        );
    }
}
