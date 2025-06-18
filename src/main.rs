mod cli;
mod config;
mod environment;
mod error;
mod http;
mod logger;

use cli::{Cli, Commands, ConfigAction};
use config::ConfigManager;
use error::AsteriskError;
use http::HttpClient;
use logger::Logger;
use colored::*;

#[tokio::main]
async fn main() -> Result<(), AsteriskError> {
    let cli = Cli::new();
    let mut config_manager = ConfigManager::new();

    // Handle subcommands first
    if let Some(command) = cli.command {
        return handle_command(command, &mut config_manager).await;
    }

    // Ensure we have endpoint and method for HTTP requests
    let endpoint = cli.endpoint.ok_or_else(|| {
        AsteriskError::Config("ENDPOINT is required for HTTP requests".to_string())
    })?;
    
    let method = cli.method.ok_or_else(|| {
        AsteriskError::Config("METHOD is required for HTTP requests".to_string())
    })?;

    // Load and resolve configuration
    let resolved_config = config_manager.load_resolved_config(
        cli.profile.as_deref(),
        cli.url.as_deref(),
        cli.token.as_deref(),
        cli.headers.as_deref(),
        cli.verbose,
    )?;

    // Initialize components
    let logger = Logger::new(resolved_config.verbose);
    let http_client = HttpClient::new();

    // Build final URL
    let url = build_request_url(&resolved_config.url, &endpoint);

    // Send request
    let (status, body, timing) = http_client
        .send_request(&url, &method, resolved_config.headers, cli.body)
        .await?;

    logger.response(status, &timing, &body);

    Ok(())
}

/// Handles configuration subcommands
async fn handle_command(command: Commands, config_manager: &mut ConfigManager) -> Result<(), AsteriskError> {
    match command {
        Commands::Config { action } => {
            match action {
                ConfigAction::Show { profile } => {
                    let config_display = config_manager.format_current_config(profile.as_deref())?;
                    
                    println!("{}", "Current Configuration:".bold().bright_cyan());
                    if let Some(config_path) = config_manager.find_config_file() {
                        println!("{} {}", "Config file:".bold(), config_path.display());
                    } else {
                        println!("{}", "Config file: None (using defaults)".italic());
                    }
                    println!();
                    print!("{}", config_display);
                }
                ConfigAction::Init => {
                    let config_path = config_manager.create_default_config()?;
                    println!("{}", "✅ Configuration initialized!".bright_green());
                    println!("{} {}", "Created:".bold(), config_path.display());
                    println!();
                    println!("{}", "Next steps:".bold());
                    println!("1. Edit {} to configure your API endpoints", config_path.display());
                    println!("2. Set environment variables for tokens (e.g., export DEV_API_KEY=your_token)");
                    println!("3. Use {} to test your configuration", "asterisk config show".bright_green());
                }
            }
        }
    }
    Ok(())
}

/// Builds the final request URL from base URL and endpoint
fn build_request_url(base_url: &str, endpoint: &str) -> String {
    let trimmed_endpoint = endpoint.trim_start_matches('/');
    
    if base_url.ends_with('/') {
        format!("{}{}", base_url, trimmed_endpoint)
    } else {
        format!("{}/{}", base_url, trimmed_endpoint)
    }
}
