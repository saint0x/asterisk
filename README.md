# Asterisk: Universal API Testing CLI

A lightweight command-line tool for rapidly testing API endpoints with a clean and simple interface.

## Installation

```bash
cargo install asterisk-cli
```

## Usage

```bash
asterisk <endpoint> <method> [options]
```

### Arguments
- `endpoint`: Endpoint name or path (e.g., "users" or "api/users")
- `method`: HTTP method (GET, POST, PUT, DELETE, PATCH)

### Options
- `-b, --body <json>`: Request body as JSON string
- `-H, --headers <headers>`: Custom headers (format: "key1:value1,key2:value2")
- `-t, --token <token>`: Bearer token for authentication
- `-u, --url <url>`: Base URL (default: http://localhost:3000)
- `-v, --verbose`: Enable detailed output

### Examples

```bash
# Basic GET request
asterisk users get

# POST with JSON body
asterisk users post -b '{"name":"John","email":"john@example.com"}'

# Authenticated request
asterisk protected get -t "your-token-here"

# Custom headers and base URL
asterisk users get -H "Accept:application/json,API-Key:123" -u "http://api.example.com"
```

## Features

- 🚀 Simple and intuitive interface
- 📝 Support for JSON payloads
- 🔒 Authentication handling
- ⚡ Request timing information
- 🎨 Beautiful terminal output

## Why Asterisk?

Asterisk is designed to be a straightforward, no-nonsense API testing tool. It focuses on doing one thing well: making HTTP requests with minimal friction. Whether you're testing a local development server or a production API, Asterisk provides a clean and efficient interface for your API testing needs.

