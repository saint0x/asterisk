use clap::{Parser, Subcommand};
use colored::*;

fn format_help() -> String {
    let mut help = String::new();
    
    // Title
    help.push_str(&format!("\n{}\n", "🌟 Asterisk CLI".bold().bright_cyan()));
    help.push_str(&format!("{}\n\n", "Universal API Testing Tool".italic()));
    
    // Usage
    help.push_str(&format!("{}\n", "USAGE:".bold().yellow()));
    help.push_str(&format!("  {} {} {}\n\n",
        "asterisk".bright_green(),
        "<ENDPOINT>".bright_blue(),
        "<METHOD>".bright_blue()
    ));
    
    // Arguments
    help.push_str(&format!("{}\n", "ARGUMENTS:".bold().yellow()));
    help.push_str(&format!("  {} {}\n", "ENDPOINT".bright_blue(), "API endpoint to test (e.g., users)"));
    help.push_str(&format!("  {} {}\n\n", "METHOD".bright_blue(), "HTTP method (GET, POST, etc.)"));
    
    // Options
    help.push_str(&format!("{}\n", "OPTIONS:".bold().yellow()));
    help.push_str(&format!("  {} {} {}\n", "-b, --body".bright_green(), "<JSON>".bright_blue(), "Request body"));
    help.push_str(&format!("  {} {} {}\n", "-H, --headers".bright_green(), "<HEADERS>".bright_blue(), "HTTP headers (key:value,...)"));
    help.push_str(&format!("  {} {} {}\n", "-t, --token".bright_green(), "<TOKEN>".bright_blue(), "Bearer token"));
    help.push_str(&format!("  {} {} {}\n", "-u, --url".bright_green(), "<URL>".bright_blue(), "Base URL [default: http://localhost:3000]"));
    help.push_str(&format!("  {} {}\n", "-v, --verbose".bright_green(), "Enable detailed output"));
    help.push_str(&format!("  {} {}\n\n", "-h, --help".bright_green(), "Show this help message"));
    
    // Examples
    help.push_str(&format!("{}\n", "EXAMPLES:".bold().yellow()));
    help.push_str(&format!("  {} {}\n", "Basic request:".bold(), "asterisk users get"));
    help.push_str(&format!("  {} {}\n", "With body:".bold(), "asterisk sign-up post -b '{\"name\":\"john\"}'"));
    help.push_str(&format!("  {} {}\n", "With token:".bold(), "asterisk users get -t 'mytoken'"));
    help.push_str(&format!("  {} {}\n", "Custom URL:".bold(), "asterisk users get -u 'https://api.example.com'"));
    
    help
}

#[derive(Parser)]
#[command(
    name = "asterisk",
    version,
    about = "🌟 Universal API Testing Tool",
    long_about = None,
    help_template = "", // Disable default help
    before_help = "",
    after_help = ""
)]
pub struct Cli {
    /// API endpoint to test
    #[arg(value_name = "ENDPOINT")]
    pub endpoint: String,

    /// HTTP method (GET, POST, PUT, etc.)
    #[arg(value_name = "METHOD")]
    pub method: String,

    /// JSON request body
    #[arg(short, long)]
    pub body: Option<String>,

    /// HTTP headers (key:value,key2:value2)
    #[arg(short = 'H', long)]
    pub headers: Option<String>,

    /// Bearer token
    #[arg(short, long)]
    pub token: Option<String>,

    /// Base URL
    #[arg(short, long, default_value = "http://localhost:3000")]
    pub url: String,

    /// Enable detailed output
    #[arg(short, long)]
    pub verbose: bool,
}

impl Cli {
    pub fn new() -> Self {
        // If help flag is present, print our custom help
        if std::env::args().any(|arg| arg == "-h" || arg == "--help") {
            println!("{}", format_help());
            std::process::exit(0);
        }
        
        Self::parse()
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// List available endpoints
    List {
        /// Filter by HTTP method
        #[arg(short, long)]
        method: Option<String>,
    },
    /// Show endpoint details
    Info {
        endpoint: String,
    },
}
