//! TaskLock - 任务锁模块
//!
//! 支持挤出机制（优雅退出）的任务锁
//! - 新实例：清空锁文件，等待旧实例退出
//! - 旧实例：检测PID变了就自己退出

mod task_lock;
mod task_timer;

pub use task_lock::TaskLock;
pub use task_timer::TaskTimer;
