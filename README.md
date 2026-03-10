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

## MyLogger - Detailed Usage

### Log Levels

| Level | Value | Description | Production | Development |
|-------|-------|-------------|------------|-------------|
| `Detail` | 5 | Detailed debugging info (AI-friendly) | No output | File only |
| `Debug` | 10 | Debug information | No output | Console + File |
| `Info` | 20 | General information | Console + File | Console + File |
| `Warn` | 30 | Warning messages | Console + File | Console + File |
| `Error` | 40 | Error messages | Console + File | Console + File |

### Environment Control

Set via environment variable `APP_ENV`:
- `production` - Only Info/Warn/Error to console and file
- `development` - All levels including Detail to file, Debug+ to console
- `test` - Same as development

```bash
# Production mode
APP_ENV=production ./your_app

# Development mode (default)
APP_ENV=development ./your_app
```

### Detail Logs for AI Debugging

```rust
use base::{MyLogger, get_logger};
use std::sync::Arc;

// Create logger with 3-day retention
let logger = MyLogger::new("my_workflow", 3);

// Detail logs are perfect for AI-assisted debugging
// They won't pollute production logs, but help during development
logger.detail("Step 1: Initializing database connection");
logger.detail("Step 2: Loading configuration from /etc/app/config.yaml");
logger.detail("Step 3: Found 42 records in cache");
logger.detail("Step 4: Processing batch size = 100");
logger.error("Failed to connect to database: Connection refused");

// In production: only Error shows
// In development: all logs written to detail.log for AI analysis
```

### Singleton Pattern with Macro

```rust
use base::mylogger;
use std::sync::Arc;

struct MyCapability {
    logger: Arc<MyLogger>,
}

impl MyCapability {
    pub fn new() -> Self {
        Self {
            logger: mylogger!(),  // Auto-detects "MyCapability" as name
        }
    }
}
```

### Log Files

- `logs/project/project.log` - Global log (Info+)
- `logs/project/detail.log` - Detailed log (Detail level, dev only)
- `logs/{name}/{name}.log` - Per-module log

## Quick Usage

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
