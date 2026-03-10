//! ProjectPath - 项目路径查找工具
//!
//! 向上查找 docs 或 .claude 目录，确定项目根目录

mod project_path;

pub use project_path::{ProjectPath, Environment};