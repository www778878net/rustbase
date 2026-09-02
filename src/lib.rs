//! Base - 基础库
//!
//! 提供日志、项目路径等通用组件
//!
//! ## 模块结构
//! - `mylogger`: 日志组件
//! - `upinfo`: 请求上下文
//! - `project_path`: 项目路径工具
//! - `frontmatter`: FrontMatter 解析
//! - `task_lock`: 任务锁（挤出机制）
//!
//! 说明：HTTP 客户端已拆分到平台专用入口（国内 `svccn`、海外 `svccf`），
//! 本库保持纯化、无网络依赖，可编译为 wasm。

// 声明子模块
pub mod frontmatter;
pub mod mylogger;
pub mod project_path;
pub mod task_lock;
pub mod upinfo;

// 重导出常用类型（方便外部直接用）

// MyLogger
pub use mylogger::{MyLogger, LogLevel, Environment, get_logger};

// ProjectPath
pub use project_path::{ProjectPath, load_ini_from_path, parse_ini_content};

// UpInfo
pub use upinfo::{RawRequest, Response, UpInfo, UpInfoError};

// FrontMatter
pub use frontmatter::{FrontMatter, TaskInfo};

// TaskLock
pub use task_lock::{TaskLock, TaskTimer};