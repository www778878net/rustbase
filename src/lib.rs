//! Base - 基础库
//!
//! 提供日志、HTTP、项目路径等通用组件
//!
//! ## 模块结构
//! - `mylogger`: 日志组件
//! - `upinfo`: 请求上下文
//! - `http`: HTTP 工具
//! - `project_path`: 项目路径工具
//! - `frontmatter`: FrontMatter 解析
//! - `task_lock`: 任务锁（挤出机制）

// 声明子模块
pub mod frontmatter;
pub mod http;
pub mod mylogger;
pub mod project_path;
pub mod task_lock;
pub mod upinfo;

// 重导出常用类型（方便外部直接用）

// MyLogger
pub use mylogger::{MyLogger, LogLevel, Environment, get_logger};

// HTTP
pub use http::{HttpHelper, HttpResponse, ResponseData};

// ProjectPath
pub use project_path::ProjectPath;

// UpInfo
pub use upinfo::{UpInfo, UpInfoError, Response};

// FrontMatter
pub use frontmatter::{FrontMatter, TaskInfo};

// TaskLock
pub use task_lock::{TaskLock, TaskTimer};