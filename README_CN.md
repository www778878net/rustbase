# Base

Rust 基础库，提供日志、HTTP、项目路径等通用组件。

[English](./README.md)

## 功能模块

- **MyLogger** - 结构化日志，支持多级别和文件输出
- **HttpHelper** - HTTP 客户端工具，支持 JSON
- **ProjectPath** - 跨平台项目路径管理
- **FrontMatter** - YAML Front Matter 解析和渲染
- **TaskLock** - 基于文件的任务锁机制
- **UpInfo** - 请求上下文和响应处理

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
base = { git = "https://github.com/www778878net/rustbase.git" }
```

## 使用示例

```rust
use base::{MyLogger, HttpHelper, ProjectPath, FrontMatter, TaskLock};

// 日志
let logger = MyLogger::new("my_app", 3);
logger.detail("详细信息");
logger.error("错误信息");

// HTTP
let http = HttpHelper::new("https://api.example.com");
let response = http.get("/users")?;

// FrontMatter
let fm = FrontMatter::parse(&content)?;
let rendered = fm.render("正文内容");

// 任务锁
let lock = TaskLock::new("my_task");
if lock.acquire() {
    // 执行任务
    lock.release();
}
```

## 模块说明

| 模块 | 说明 |
|------|------|
| `mylogger` | 结构化日志，支持文件输出和日志轮转 |
| `http` | HTTP 客户端封装，支持 JSON |
| `project_path` | 项目路径工具，跨平台兼容 |
| `frontmatter` | YAML Front Matter 解析和渲染 |
| `task_lock` | 基于文件的任务锁，支持挤出机制 |
| `upinfo` | 请求上下文和响应数据结构 |

## 依赖

- `chrono` - 日期时间
- `parking_lot` - 同步原语
- `serde` / `serde_json` / `serde_yaml` - 序列化
- `ureq` - HTTP 客户端
- `uuid` - UUID 生成
- `base64` - Base64 编码
- `regex` - 正则表达式

## 许可证

MIT 许可证 - 详见 [LICENSE](./LICENSE)

## 仓库地址

- GitHub: https://github.com/www778878net/rustbase
- CNB: https://cnb.cool/778878/rustbase
