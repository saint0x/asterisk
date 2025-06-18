# Asterisk: Universal API Testing CLI

A lightweight command-line tool for rapidly testing API endpoints with project-level configuration support.

## Installation

```bash
cargo install asterisk-cli
```

## Quick Start

```bash
# Basic request
asterisk users get

# Initialize project configuration
asterisk config init

# Use environment profiles
asterisk users get --profile staging
```

## Configuration

Create an `asterisk.config` file in your project:

```toml
default_profile = "dev"

[profiles.dev]
url = "http://localhost:3000"
token = "$DEV_API_KEY"

[profiles.staging]
url = "https://staging-api.example.com"
token = "$STAGING_TOKEN"

[profiles.production]
url = "https://api.example.com"
token = "$PROD_API_KEY"
```

## Usage

### Basic Commands
```bash
# Use default profile from config
asterisk users get

# Switch environments
asterisk users get --profile production

# POST with body
asterisk users post -b '{"name":"John"}'

# Override config settings
asterisk users get -u "http://localhost:8080"
```

### Configuration Management
```bash
# Create config template
asterisk config init

# View current settings
asterisk config show
asterisk config show --profile staging
```

### Options
- `-p, --profile <name>`: Use specific environment profile
- `-u, --url <url>`: Override base URL
- `-t, --token <token>`: Override bearer token
- `-b, --body <json>`: Request body as JSON
- `-H, --headers <headers>`: Custom headers (`key:value,key2:value2`)
- `-v, --verbose`: Detailed output

## Features

- 🚀 **Simple CLI** - Clean, intuitive interface
- ⚙️ **Project Config** - TOML-based configuration with environment profiles
- 🔒 **Secure Tokens** - Environment variable references (`$API_KEY`)
- ⚡ **Fast** - Request timing and performance metrics
- 🎨 **Beautiful Output** - Colored status codes and formatted JSON

