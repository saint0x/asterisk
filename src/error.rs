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
