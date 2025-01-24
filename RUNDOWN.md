# Asterisk: Technical Rundown

## Architecture Overview

Asterisk is a Rust CLI application that provides a clean interface for making HTTP requests. It focuses on simplicity and efficiency, making it easy to test APIs with minimal configuration.

## Core Components

### 1. CLI Interface
- **clap**: Command-line argument parsing with structured type definitions
- **colored**: Terminal output formatting and coloring

### 2. HTTP Client 
- **reqwest**: Async HTTP client for making API requests
- **tokio**: Async runtime for handling HTTP operations
- **serde_json**: JSON serialization/deserialization

### 3. Error Handling
- **thiserror**: Error type definitions and handling

## How It Works

### 1. Request Building
```rust
fn send_request(url: &str, method: &str, headers: HeaderMap, body: Option<String>) -> Result<Response, Error>
```
- Constructs HTTP request with provided parameters
- Handles headers and authentication
- Processes request body

### 2. Response Handling
```rust
fn response(status: u16, timing: &RequestTiming, body: &str)
```
- Formats response output
- Shows status code and timing
- Pretty-prints JSON when possible

## Key Features Implementation

### HTTP Request Handling
- Support for all standard HTTP methods
- Custom header processing
- Bearer token authentication
- JSON body handling

### Response Formatting
- Status code coloring
- Request timing information
- JSON pretty printing
- Verbose output option

## Dependencies Explained

### Core
- **clap**: Modern argument parser with derive macros
- **reqwest**: Feature-rich HTTP client with async support
- **tokio**: Efficient async runtime for I/O operations

### Utility
- **serde_json**: JSON support
- **colored**: ANSI color formatting

### Error Management
- **thiserror**: Custom error types

## Performance Considerations

- Async I/O operations
- Minimal memory footprint
- Fast binary startup time
- Efficient header parsing

## Security Features

- No persistent data storage
- Optional Bearer token support
- Header validation
- Safe JSON parsing

## 🔄 Technical Data Flow

### 1. CLI Argument Processing
1. User invokes `asterisk` with arguments (e.g., `asterisk users get`)
2. `main.rs` calls `Cli::new()` which:
   - Checks for help flag using `std::env::args()`
   - If help flag present, renders colorful help menu via `format_help()`
   - Otherwise, uses `clap` to parse arguments into `Cli` struct

### 2. Request Building
1. `main.rs` creates instances of core components:
   ```rust
   let logger = Logger::new(cli.verbose);
   let http_client = HttpClient::new();
   ```
2. Processes headers and token:
   ```rust
   let headers = HttpClient::parse_headers(headers_str, token)?;
   ```

### 3. URL Construction
1. Builds the final URL:
   ```rust
   let url = if cli.url.ends_with('/') {
       format!("{}{}", cli.url, endpoint.trim_start_matches('/'))
   } else {
       format!("{}/{}", cli.url, endpoint.trim_start_matches('/'))
   };
   ```

### 4. Request Execution
1. `HttpClient` sends request:
   ```rust
   let (status, body, timing) = http_client
       .send_request(&url, &cli.method, headers, body)
       .await?;
   ```
2. Measures request timing:
   - Records start time
   - Tracks total execution time
   - Formats timing information

### 5. Response Processing
1. `Logger` formats and displays response:
   ```rust
   logger.response(status, &timing, &body);
   ```
2. In verbose mode:
   - Shows detailed status information
   - Pretty-prints JSON responses
   - Displays timing metrics
3. In normal mode:
   - Shows concise status code
   - Displays raw response
   - Shows basic timing

### 6. Error Handling
1. Uses custom `AsteriskError` enum for all errors:
   ```rust
   pub enum AsteriskError {
       Io(std::io::Error),
       Http(reqwest::Error),
       InvalidHeaders,
   }
   ```
2. Error propagation through `Result` types
3. Friendly error messages via `thiserror` derive macro

## 📖 Command Manual

```
🌟 Asterisk CLI
Universal API Testing Tool

USAGE:
  asterisk <ENDPOINT> <METHOD>

ARGUMENTS:
  ENDPOINT     API endpoint to test (e.g., users)
  METHOD       HTTP method (GET, POST, etc.)

OPTIONS:
  -b, --body <JSON>       Request body
  -H, --headers <HEADERS> HTTP headers (key:value,...)
  -t, --token <TOKEN>     Bearer token
  -u, --url <URL>         Base URL [default: http://localhost:3000]
  -v, --verbose           Enable detailed output
  -h, --help             Show this help message

EXAMPLES:
  Basic request:     asterisk users get
  With body:         asterisk sign-up post -b '{"name":"john"}'
  With token:        asterisk users get -t 'mytoken'
  Custom URL:        asterisk health get -u "https://api.example.com"
