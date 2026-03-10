# Base

A foundational Rust library providing common utilities for logging, HTTP, project paths, and more.

[中文文档](./README_CN.md)

## Features

- **MyLogger** - Structured logging with multiple levels and file output
- **HttpHelper** - HTTP client utilities with JSON support
- **ProjectPath** - Cross-platform project path management
- **FrontMatter** - YAML front matter parser and renderer
- **TaskLock** - File-based task locking mechanism
- **UpInfo** - Request context and response handling

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
base = { git = "https://github.com/www778878net/rustbase.git" }
```

## Usage

```rust
use base::{MyLogger, HttpHelper, ProjectPath, FrontMatter, TaskLock};

// Logging
let logger = MyLogger::new("my_app", 3);
logger.detail("Detailed message");
logger.error("Error message");

// HTTP
let http = HttpHelper::new("https://api.example.com");
let response = http.get("/users")?;

// FrontMatter
let fm = FrontMatter::parse(&content)?;
let rendered = fm.render("Body content");

// TaskLock
let lock = TaskLock::new("my_task");
if lock.acquire() {
    // Do work
    lock.release();
}
```

## Modules

| Module | Description |
|--------|-------------|
| `mylogger` | Structured logging with file output and log rotation |
| `http` | HTTP client wrapper with JSON support |
| `project_path` | Project path utilities for cross-platform compatibility |
| `frontmatter` | YAML front matter parsing and rendering |
| `task_lock` | File-based task locking with eviction mechanism |
| `upinfo` | Request context and response data structures |

## Dependencies

- `chrono` - Date and time
- `parking_lot` - Synchronization primitives
- `serde` / `serde_json` / `serde_yaml` - Serialization
- `ureq` - HTTP client
- `uuid` - UUID generation
- `base64` - Base64 encoding
- `regex` - Regular expressions

## License

Apache License 2.0 - see [LICENSE](./LICENSE) for details.

## Repository

- GitHub: https://github.com/www778878net/rustbase
- CNB: https://cnb.cool/778878/rustbase
