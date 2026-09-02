//! TaskLock - 任务锁实现
//!
//! 支持挤出机制（优雅退出）
//! 锁文件位置: tmp/lockid/{name}.lock
//! 新实例：清空锁文件，等待旧实例退出
//! 旧实例：检测PID变了就自己退出

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::project_path::ProjectPath;

/// 任务锁 - 支持挤出机制（优雅退出）
pub struct TaskLock {
    lock_path: PathBuf,
    pid: u32,
    running: Arc<AtomicBool>,
}

impl TaskLock {
    /// 创建任务锁
    /// 
    /// # Arguments
    /// * `name` - 锁名称，用于区分不同的任务
    /// 
    /// # Returns
    /// TaskLock 实例
    pub fn new(name: &str) -> Self {
        let project_path = ProjectPath::find().unwrap_or_default();
        let lock_path = project_path.join("tmp/lockid").join(format!("{}.lock", name));
        let pid = std::process::id();
        
        Self { 
            lock_path, 
            pid,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// 创建任务锁（指定锁文件路径）
    /// 
    /// # Arguments
    /// * `lock_path` - 锁文件路径
    /// 
    /// # Returns
    /// TaskLock 实例
    pub fn with_path(lock_path: PathBuf) -> Self {
        let pid = std::process::id();
        
        Self { 
            lock_path, 
            pid,
            running: Arc::new(AtomicBool::new(true)),
        }
    }
    
    /// 尝试获取锁（挤出旧实例）
    /// 
    /// 新实例：清空锁文件，等待旧实例自己退出
    /// 
    /// # Returns
    /// * `Ok(true)` - 获取锁成功
    /// * `Err(...)` - 获取锁失败
    pub fn try_acquire(&self) -> Result<bool, String> {
        if !self.lock_path.exists() {
            return self.create_lock();
        }
        
        let content = match std::fs::read_to_string(&self.lock_path) {
            Ok(c) => c,
            Err(_) => return self.create_lock(),
        };
        
        let old_pid: u32 = match content.trim().parse() {
            Ok(p) => p,
            Err(_) => return self.create_lock(),
        };
        
        if old_pid == self.pid {
            return Ok(true);
        }
        
        // 清空锁文件，旧实例检测到会自己退出
        let _ = std::fs::remove_file(&self.lock_path);
        
        // 等待旧实例退出
        self.wait_old_exit(old_pid);
        
        self.create_lock()
    }

    /// 尝试获取锁（带日志输出）
    /// 
    /// # Arguments
    /// * `log_fn` - 日志输出函数
    /// 
    /// # Returns
    /// * `Ok(true)` - 获取锁成功
    /// * `Err(...)` - 获取锁失败
    pub fn try_acquire_with_log<F>(&self, log_fn: F) -> Result<bool, String>
    where
        F: Fn(&str),
    {
        if !self.lock_path.exists() {
            return self.create_lock();
        }
        
        let content = match std::fs::read_to_string(&self.lock_path) {
            Ok(c) => c,
            Err(_) => return self.create_lock(),
        };
        
        let old_pid: u32 = match content.trim().parse() {
            Ok(p) => p,
            Err(_) => return self.create_lock(),
        };
        
        if old_pid == self.pid {
            return Ok(true);
        }
        
        // 清空锁文件，旧实例检测到会自己退出
        log_fn(&format!("[LOCK] 发现旧实例 (PID: {})，正在挤出...", old_pid));
        let _ = std::fs::remove_file(&self.lock_path);
        
        // 等待旧实例退出
        self.wait_old_exit_with_log(old_pid, &log_fn);
        
        self.create_lock()
    }
    
    /// 等待旧实例退出
    fn wait_old_exit(&self, old_pid: u32) {
        for _ in 1..=10 {
            if !self.is_process_alive(old_pid) {
                return;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    /// 等待旧实例退出（带日志）
    fn wait_old_exit_with_log<F>(&self, old_pid: u32, log_fn: &F)
    where
        F: Fn(&str),
    {
        for i in 1..=10 {
            if !self.is_process_alive(old_pid) {
                log_fn("[LOCK] 旧实例已退出");
                return;
            }
            log_fn(&format!("[LOCK] 等待旧实例退出... ({}/10)", i));
            std::thread::sleep(Duration::from_millis(500));
        }
        log_fn("[LOCK] 旧实例未退出，继续执行");
    }
    
    /// 创建锁文件
    fn create_lock(&self) -> Result<bool, String> {
        if let Some(parent) = self.lock_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Err(format!("create lock dir failed: {}", e));
            }
        }
        
        if let Err(e) = std::fs::write(&self.lock_path, self.pid.to_string()) {
            return Err(format!("write lock file failed: {}", e));
        }
        
        Ok(true)
    }
    
    /// 释放锁
    pub fn release(&self) -> Result<(), String> {
        self.running.store(false, Ordering::SeqCst);
        if self.lock_path.exists() {
            // 只有锁文件里的PID是自己时才删除
            if let Ok(content) = std::fs::read_to_string(&self.lock_path) {
                if content.trim() == self.pid.to_string() {
                    if let Err(e) = std::fs::remove_file(&self.lock_path) {
                        return Err(format!("remove lock file failed: {}", e));
                    }
                }
            }
        }
        Ok(())
    }
    
    /// 检查是否被挤出（PID变了）
    /// 
    /// 在循环中定期调用此方法，检测是否被新实例挤出
    /// 
    /// # Returns
    /// * `true` - 被挤出，应该退出
    /// * `false` - 未被挤出，继续执行
    pub fn check_kicked(&self) -> bool {
        if !self.lock_path.exists() {
            return true;
        }

        if let Ok(content) = std::fs::read_to_string(&self.lock_path) {
            let current_pid = content.trim();
            if current_pid != self.pid.to_string() {
                return true;
            }
        }

        false
    }
    
    /// 获取运行标志
    /// 
    /// 可以在循环中检查此标志，当被挤出时标志会变为false
    pub fn running_flag(&self) -> Arc<AtomicBool> {
        self.running.clone()
    }

    /// 获取锁文件路径
    pub fn lock_path(&self) -> &PathBuf {
        &self.lock_path
    }

    /// 获取当前PID
    pub fn pid(&self) -> u32 {
        self.pid
    }
    
    /// 检查进程是否存活
    #[cfg(windows)]
    fn is_process_alive(&self, pid: u32) -> bool {
        use std::process::Command;
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout).contains(&pid.to_string())
            })
            .unwrap_or(false)
    }
    
    #[cfg(not(windows))]
    fn is_process_alive(&self, pid: u32) -> bool {
        use std::path::Path;
        Path::new("/proc").join(pid.to_string()).exists()
    }
}

impl Drop for TaskLock {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_lock_new() {
        let lock = TaskLock::new("test_lock");
        assert!(lock.lock_path().to_string_lossy().contains("test_lock.lock"));
    }

    #[test]
    fn test_task_lock_acquire_release() {
        let lock = TaskLock::new("test_lock_acquire");
        
        // 清理可能存在的旧锁
        let _ = std::fs::remove_file(lock.lock_path());
        
        // 获取锁
        let result = lock.try_acquire();
        assert!(result.is_ok());
        
        // 检查锁文件存在
        assert!(lock.lock_path().exists());
        
        // 释放锁
        let result = lock.release();
        assert!(result.is_ok());
        
        // 检查锁文件不存在
        assert!(!lock.lock_path().exists());
    }

    #[test]
    fn test_check_kicked() {
        let lock = TaskLock::new("test_lock_kicked");
        
        // 清理可能存在的旧锁
        let _ = std::fs::remove_file(lock.lock_path());
        
        // 获取锁
        let _ = lock.try_acquire();
        
        // 未被挤出
        assert!(!lock.check_kicked());
        
        // 模拟被挤出：修改锁文件内容
        std::fs::write(lock.lock_path(), "99999").unwrap();
        
        // 被挤出
        assert!(lock.check_kicked());
        
        // 清理
        let _ = std::fs::remove_file(lock.lock_path());
    }
}
