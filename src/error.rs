use thiserror::Error;
use reqwest::Error as ReqwestError;

pub type Result<T> = std::result::Result<T, AsteriskError>;

#[derive(Error, Debug)]
pub enum AsteriskError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("{}", format_http_error(.0))]
    Http(#[from] ReqwestError),
    
    #[error("Invalid headers format")]
    InvalidHeaders,
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Missing required environment variable: {0}")]
    MissingEnvironmentVariable(String),
    
    #[error("Invalid environment variable: {0}")]
    InvalidEnvironmentVariable(String),
    
    #[error("TOML parsing error: {0}")]
    TomlParsing(#[from] toml::de::Error),
}

fn format_http_error(err: &ReqwestError) -> String {
    if err.is_connect() {
        format!("Could not connect to server. Is it running? [{}]",
            err.url().map(|u| u.as_str()).unwrap_or("unknown URL"))
    } else if err.is_timeout() {
        format!("Request timed out. Server might be slow or unresponsive [{}]",
            err.url().map(|u| u.as_str()).unwrap_or("unknown URL"))
    } else {
        err.to_string()
    }
}
