use std::env;

#[derive(Debug)]
pub struct Config {
    /// Defaults to "0.0.0.0:3000"
    pub socket: String,
    /// Defaults to "sqlite://./db.sqlite?mode=rwc"
    pub db_path: String,
    #[cfg(feature = "fireframe")]
    pub fireframe_socket: String,
}

impl Config {
    pub fn get() -> Self {
        Config {
            socket: env::var("SOCKET").unwrap_or("0.0.0.0:3000".into()),
            db_path: env::var("DB_PATH").unwrap_or("sqlite://./db.sqlite?mode=rwc".into()),
            #[cfg(feature = "fireframe")]
            fireframe_socket: env::var("FF_SOCK").unwrap_or("./ff.sock".into()),
        }
    }
}
