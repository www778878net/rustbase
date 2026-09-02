//! TaskTimer - 任务定时器
//!
//! 用于控制任务的执行频率，记录定时任务的最后执行时间
//! 文件位置: tmp/lockid/{name}.time
//!
//! 与 TaskLock 的区别：
//! - TaskLock: 进程互斥锁，防止同一任务多实例并发运行
//! - TaskTimer: 时间间隔控制器，控制任务执行频率
//!
//! 典型用法：
//! ```ignore
//! let timer = TaskTimer::new("sync_task");
//! if timer.should_run(3600) {  // 每小时执行一次
//!     // 执行任务...
//!     timer.mark_done()?;  // 记录完成时间
//! }
//! ```

use std::path::PathBuf;

use crate::project_path::ProjectPath;

/// 任务定时器 - 控制任务执行频率
///
/// 通过记录最后执行时间的时间戳，判断是否应该执行任务
pub struct TaskTimer {
    time_path: PathBuf,
}

impl TaskTimer {
    /// 创建任务定时器
    /// 
    /// 时间文件位置: tmp/lockid/{name}.time
    /// 
    /// # Arguments
    /// * `name` - 定时器名称，用于区分不同的定时任务
    /// 
    /// # Returns
    /// TaskTimer 实例
    pub fn new(name: &str) -> Self {
        let project_path = ProjectPath::find().unwrap_or_default();
        let time_path = project_path.join("tmp/lockid").join(format!("{}.time", name));
        
        Self { time_path }
    }

    /// 创建任务定时器（指定时间文件路径）
    /// 
    /// # Arguments
    /// * `relative_path` - 相对于项目根目录的时间文件路径
    /// 
    /// # Returns
    /// TaskTimer 实例
    pub fn with_path(relative_path: &str) -> Self {
        let project_path = ProjectPath::find().unwrap_or_default();
        let time_path = project_path.join(relative_path);
        
        Self { time_path }
    }

    /// 检查是否应该执行任务
    /// 
    /// 判断距离上次执行是否已超过指定间隔
    /// 
    /// # Arguments
    /// * `interval_secs` - 执行间隔（秒）
    /// 
    /// # Returns
    /// * `true` - 应该执行（首次执行或已超过间隔）
    /// * `false` - 不应该执行（未超过间隔）
    pub fn should_run(&self, interval_secs: u64) -> bool {
        if !self.time_path.exists() {
            return true;
        }
        
        let content = match std::fs::read_to_string(&self.time_path) {
            Ok(c) => c,
            Err(_) => return true,
        };
        
        let last_time: u64 = match content.trim().parse() {
            Ok(t) => t,
            Err(_) => return true,
        };
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        now - last_time >= interval_secs
    }

    /// 标记任务完成
    /// 
    /// 将当前时间戳写入时间文件，用于下次判断执行间隔
    /// 
    /// # Returns
    /// * `Ok(())` - 标记成功
    /// * `Err(...)` - 写入失败
    pub fn mark_done(&self) -> Result<(), String> {
        if let Some(parent) = self.time_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Err(format!("create dir failed: {}", e));
            }
        }
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        if let Err(e) = std::fs::write(&self.time_path, now.to_string()) {
            return Err(format!("write time file failed: {}", e));
        }
        
        Ok(())
    }

    /// 获取上次执行时间
    /// 
    /// # Returns
    /// * `Some(u64)` - 上次执行的 Unix 时间戳（秒）
    /// * `None` - 从未执行过或读取失败
    pub fn get_last_time(&self) -> Option<u64> {
        if !self.time_path.exists() {
            return None;
        }
        
        let content = std::fs::read_to_string(&self.time_path).ok()?;
        content.trim().parse().ok()
    }

    /// 获取时间文件路径
    pub fn time_path(&self) -> &PathBuf {
        &self.time_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_timer_new() {
        let timer = TaskTimer::new("test_timer_new");
        assert!(timer.time_path().to_string_lossy().contains("test_timer_new.time"));
        // 清理
        let _ = std::fs::remove_file(timer.time_path());
    }

    #[test]
    fn test_task_timer_first_run() {
        let timer = TaskTimer::new("test_timer_first");
        let _ = std::fs::remove_file(timer.time_path());

        // 首次执行，应该运行
        assert!(timer.should_run(3600));

        // 清理
        let _ = std::fs::remove_file(timer.time_path());
    }

    #[test]
    fn test_task_timer_mark_done() {
        let timer = TaskTimer::new("test_timer_mark");
        let _ = std::fs::remove_file(timer.time_path());

        // 标记完成
        let result = timer.mark_done();
        assert!(result.is_ok());

        // 检查文件创建
        assert!(timer.time_path().exists());

        // 清理
        let _ = std::fs::remove_file(timer.time_path());
    }

    #[test]
    fn test_task_timer_interval_check() {
        let timer = TaskTimer::new("test_timer_interval");
        let _ = std::fs::remove_file(timer.time_path());

        // 首次执行
        assert!(timer.should_run(3600));

        // 标记完成
        timer.mark_done().unwrap();

        // 刚完成，不应该运行
        assert!(!timer.should_run(3600));

        // 清理
        let _ = std::fs::remove_file(timer.time_path());
    }

    #[test]
    fn test_task_timer_get_last_time() {
        let timer = TaskTimer::new("test_timer_last_time");
        let _ = std::fs::remove_file(timer.time_path());

        // 从未执行过
        assert!(timer.get_last_time().is_none());

        // 标记完成
        timer.mark_done().unwrap();

        // 有执行记录
        let last_time = timer.get_last_time();
        assert!(last_time.is_some());
        assert!(last_time.unwrap() > 0);

        // 清理
        let _ = std::fs::remove_file(timer.time_path());
    }

    #[test]
    fn test_task_timer_with_path() {
        let timer = TaskTimer::with_path("tmp/test_custom_timer.time");

        // 确保路径正确
        let path = timer.time_path();
        assert!(path.to_string_lossy().contains("tmp/test_custom_timer.time"));

        // 清理
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_task_timer_zero_interval() {
        let timer = TaskTimer::new("test_timer_zero");
        let _ = std::fs::remove_file(timer.time_path());

        // 标记完成
        timer.mark_done().unwrap();

        // 0秒间隔应该总是返回 true
        assert!(timer.should_run(0));

        // 清理
        let _ = std::fs::remove_file(timer.time_path());
    }

    #[test]
    fn test_task_timer_overwrite() {
        let timer = TaskTimer::new("test_timer_overwrite");
        let _ = std::fs::remove_file(timer.time_path());

        // 第一次标记
        timer.mark_done().unwrap();
        let time1 = timer.get_last_time().unwrap();

        // 等待一小段时间
        std::thread::sleep(std::time::Duration::from_millis(100));

        // 第二次标记（覆盖）
        timer.mark_done().unwrap();
        let time2 = timer.get_last_time().unwrap();

        // 时间应该更新
        assert!(time2 >= time1);

        // 清理
        let _ = std::fs::remove_file(timer.time_path());
    }
}
