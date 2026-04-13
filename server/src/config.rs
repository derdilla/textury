use std::env;

#[derive(Debug)]
pub struct Config {
    /// Defaults to "0.0.0.0:3000"
    pub socket: String,
    /// Defaults to "sqlite://./db.sqlite?mode=rwc"
    pub db_path: String,
    /// Defaults to "./projects/"
    pub projects_root: String,
    /// Defaults to true
    pub sandbox_compilations: bool,
}

impl Config {
    pub fn get() -> Self {
        Config {
            socket: env::var("SOCKET").unwrap_or("0.0.0.0:3000".into()),
            db_path: env::var("DB_PATH").unwrap_or("sqlite://./db.sqlite?mode=rwc".into()),
            projects_root: env::var("PROJECTS_ROOT").unwrap_or("./projects/".into()),
            sandbox_compilations: &env::var("SANDBOX_COMPILATIONS").unwrap_or("true".into()) != "false",
        }
    }
}
