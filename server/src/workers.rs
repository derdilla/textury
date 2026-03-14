use bollard::config::ContainerCreateBody;
use bollard::query_parameters::CreateContainerOptionsBuilder;
use std::collections::HashMap;
use std::convert::Into;
use std::sync::{Arc, RwLock};
use tracing::{debug, error};

const WORKER_IMAGE: &str = "worker";

struct Worker {
    dir: tempfile::TempDir,
    client: reqwest::Client,
}

#[derive(Clone)]
pub struct WorkerManager(Arc<RwLock<HashMap<String, Arc<Worker>>>>);

impl WorkerManager {
    pub fn new() -> Self {
        WorkerManager(Arc::new(RwLock::new(HashMap::new())))
    }

    async fn start_worker(&self, project_id: &str) -> bool {
        // set up variables and paths
        let docker = bollard::Docker::connect_with_unix_defaults().unwrap();
        let worker_id = format!("worker_{}", &project_id);
        let dir = tempfile::tempdir().map_err(|err| err.to_string()).unwrap();
        let socket_path = dir.path().join("worker.sock");

        // create projects container
        let create_options = CreateContainerOptionsBuilder::new()
            .name(&worker_id)
            .build();
        let create_body = ContainerCreateBody {
            image: Some(WORKER_IMAGE.into()),
            volumes: Some(vec![format!(
                "{}:/var/run/host.sock",
                socket_path.to_str().unwrap()
            )]),
            // hardening:
            network_disabled: Some(true),
            ..Default::default()
        };
        let container = match docker
            .create_container(Some(create_options), create_body)
            .await
        {
            Ok(c) => c,
            Err(err) => {
                error!("Failed to create worker container {}", err);
                return false;
            }
        };
        for w in container.warnings {
            debug!("Container creation warning: {}", w);
        }
        let worker_id = container.id;

        // start container
        // TODO: constrain resources
        if let Err(e) = docker.start_container(&worker_id, None).await {
            error!("Failed to start worker {}", e);
            return false;
        }

        // create client
        let client = reqwest::Client::builder().unix_socket(socket_path).build();
        let client = match client {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to create client {}", e);
                return false;
            }
        };

        // save
        self.0
            .write()
            .unwrap()
            .insert(project_id.to_string(), Arc::new(Worker { dir, client }));
        true
    }

    /// Get http client for worker, starting it if not already running.
    pub async fn get_worker(&self, project_id: String) -> Option<reqwest::Client> {
        let contains_key = { self.0.read().unwrap().contains_key(&project_id) };
        if !contains_key {
            self.start_worker(project_id.as_str()).await;
        }
        self.0
            .read()
            .unwrap()
            .get(&project_id)
            .map(|w| w.client.clone())
    }
}
