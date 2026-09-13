#![allow(unused)]

use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::error;

/// Configurations that can't be changed after starting the server.
#[derive(Debug)]
pub struct Config {
    pub socket: String,
    pub db_path: String,
    pub fireframe_socket: Option<PathBuf>,
    pub idm_provides: Vec<IdentityProvider>,
    // TODO: compiler file paths
}

#[derive(Debug, Deserialize)]
pub struct IdentityProvider {
    pub name: String,
    pub client_id: String,
    pub issuer: String,
}

impl Config {
    pub fn resolve() -> Option<Config> {
        let cli = CLI::parse();
        let raw = match std::fs::read_to_string(&cli.config_path) {
            Ok(raw) => raw,
            Err(e) => {
                error!(
                    "Unable to read config file at {}",
                    cli.config_path
                );
                return None;
            }
        };
        let file: ConfigFile = match toml::from_str(&raw) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to parse config file: {}", e);
                return None;
            }
        };

        Some(Config {
            socket: cli.socket.or(file.socket).expect("missing socket"),
            db_path: cli
                .db_path
                .or(file.database)
                .expect("missing database config"),
            fireframe_socket: cli.fireframe_socket.or(file.fireframe_socket),
            idm_provides: file.idp.unwrap_or(Vec::new()),
        })
    }
}

/// Start the dilladocs server. ENV overrides CLI overrides config-file.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CLI {
    /// Path of the config file. Environment variables and CLI arguments take precedence for duplicate attributes.
    #[arg(short, long, env = "TEXTURY_CONFIG_PATH", default_value_t = "textury.toml".to_string())]
    config_path: String,
    /// Where to open the web-server
    #[arg(short, long, env = "TEXTURY_SOCKET")]
    socket: Option<String>,
    #[arg(short, long, env = "TEXTURY_DB_PATH")]
    db_path: Option<String>,
    /// Socket of a fireframe process to allow advanced sandboxing.
    #[arg(short, long, env = "TEXTURY_FIREFRAME_SOCKET")]
    fireframe_socket: Option<PathBuf>,
}

#[derive(Deserialize)]
struct ConfigFile {
    socket: Option<String>,
    database: Option<String>,
    fireframe_socket: Option<PathBuf>,
    idp: Option<Vec<IdentityProvider>>,
}
