//! MyLogger - 日志组件
//!
//! 工作流日志类：
//! - 全局日志：logs/project/project.log 和 logs/project/detail.log
//! - 工作流日志：logs/{wfname}/{wfname}.log 或 logs/{myname}/{myname}.log

pub mod mylogger;

pub use mylogger::{MyLogger, LogLevel, Environment, get_logger};