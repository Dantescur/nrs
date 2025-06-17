use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NrsError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("ClapError: {0}")]
    ClapError(#[from] clap::error::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Registry not found: {0}")]
    RegistryNotFound(String),
    #[error("Home directory not found")]
    HomeDirNotFound,
    #[error("Invalid registry URL: {0}")]
    InvalidRegistryUrl(String),
}
