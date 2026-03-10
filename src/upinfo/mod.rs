//! UpInfo - 请求上下文模块
//!
//! 支持 jsdata (JSON) 和 bytedata (二进制) 两种数据格式

mod upinfo;

pub use upinfo::{UpInfo, UpInfoError, Response};