# Base

Rust 基础库，提供日志、HTTP、项目路径等通用组件。

[English](./README.md)

## 功能模块

- **MyLogger** - 结构化日志，支持多级别和文件输出
- **HttpLogger** - HTTP 客户端工具，支持 JSON
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

## MyLogger - 详细使用说明

### 日志级别

| 级别 | 值 | 说明 | 生产环境 | 开发环境 |
|------|---|------|---------|---------|
| `Detail` | 5 | 详细调试信息（AI 友好） | 不输出 | 仅文件 |
| `Debug` | 10 | 调试信息 | 不输出 | 控制台+文件 |
| `Info` | 20 | 一般信息 | 控制台+文件 | 控制台+文件 |
| `Warn` | 30 | 警告信息 | 控制台+文件 | 控制台+文件 |
| `Error` | 40 | 错误信息 | 控制台+文件 | 控制台+文件 |

### 环境控制

通过环境变量 `APP_ENV` 设置：
- `production` - 仅 Info/Warn/Error 输出到控制台和文件
- `development` - 所有级别（含 Detail）写入文件，Debug+ 输出到控制台
- `test` - 与 development 相同

```bash
# 生产模式
APP_ENV=production ./your_app

# 开发模式（默认）
APP_ENV=development ./your_app
```

### Detail 日志用于 AI 调试

```rust
use base::{MyLogger, get_logger};
use std::sync::Arc;

// 创建日志器，保留 3 天
let logger = MyLogger::new("my_workflow", 3);

// Detail 日志非常适合 AI 辅助调试
// 生产环境不会输出，开发时帮助定位问题
logger.detail("步骤1: 初始化数据库连接");
logger.detail("步骤2: 从 /etc/app/config.yaml 加载配置");
logger.detail("步骤3: 缓存中发现 42 条记录");
logger.detail("步骤4: 处理批次大小 = 100");
logger.error("数据库连接失败: Connection refused");

// 生产环境: 只有 Error 显示
// 开发环境: 所有日志写入 detail.log 供 AI 分析
```

### 单例模式与宏

```rust
use base::mylogger;
use std::sync::Arc;

struct MyCapability {
    logger: Arc<MyLogger>,
}

impl MyCapability {
    pub fn new() -> Self {
        Self {
            logger: mylogger!(),  // 自动获取 "MyCapability" 作为名称
        }
    }
}
```

### 日志文件

- `logs/project/project.log` - 全局日志（Info+）
- `logs/project/detail.log` - 详细日志（Detail 级别，仅开发环境）
- `logs/{name}/{name}.log` - 模块独立日志

## 快速使用

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

Apache License 2.0 - 详见 [LICENSE](./LICENSE)

## 仓库地址

- GitHub: https://github.com/www778878net/rustbase
- CNB: https://cnb.cool/778878/rustbase
