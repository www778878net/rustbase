//! HTTP请求工具 - 提供统一的HTTP请求处理
//!
//! 功能：
//! - 代理配置
//! - 重试机制
//! - 超时处理
//! - 错误处理

mod http;

pub use http::{HttpHelper, HttpResponse, ResponseData};