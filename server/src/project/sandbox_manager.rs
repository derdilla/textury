use std::sync::Arc;

use crate::project::sandbox;

pub trait SandboxManager {
    type Sandbox: sandbox::Sandbox;

    fn preload_vm(&self, tenant_id: &str) -> impl Future<Output = Result<(), String>>;
    fn spawn(&self, tenant_id: &str) -> impl Future<Output = Result<Arc<Self::Sandbox>, String>>;
    fn kill_vm(&self, tenant_id: &str) -> impl Future<Output = Result<(), String>>;
}
