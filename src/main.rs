mod cli;
mod error;
mod http;
mod logger;

use cli::Cli;
use error::AsteriskError;
use http::HttpClient;
use logger::Logger;
use reqwest::header::HeaderMap;

#[tokio::main]
async fn main() -> Result<(), AsteriskError> {
    let cli = Cli::new();
    let logger = Logger::new(cli.verbose);
    let http_client = HttpClient::new();

    // Process headers
    let headers = if let Some(headers_str) = cli.headers.as_deref() {
        HttpClient::parse_headers(headers_str, cli.token.as_deref())?
    } else {
        HeaderMap::new()
    };

    // Build URL
    let url = if cli.url.ends_with('/') {
        format!("{}{}", cli.url, cli.endpoint.trim_start_matches('/'))
    } else {
        format!("{}/{}", cli.url, cli.endpoint.trim_start_matches('/'))
    };

    // Send request
    let (status, body, timing) = http_client
        .send_request(&url, &cli.method, headers, cli.body)
        .await?;

    logger.response(status, &timing, &body);

    Ok(())
}
