pub mod migrations;
pub mod queries;

pub use queries::{get_repo_path, write_widget_data};

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::Connection;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Invalid(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn overblick_data_dir() -> Result<std::path::PathBuf, String> {
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| "Could not determine data_local_dir".to_string())?;
    Ok(data_dir.join("overblick"))
}

pub fn open_db() -> Result<Connection, AppError> {
    let db_dir = overblick_data_dir()
        .map_err(AppError::Invalid)?;
    std::fs::create_dir_all(&db_dir)?;
    let db_path = db_dir.join("overblick.db");
    let conn = Connection::open(&db_path)?;
    migrations::run(&conn)?;
    Ok(conn)
}

pub struct AppState {
    pub db: Mutex<Connection>,
}
