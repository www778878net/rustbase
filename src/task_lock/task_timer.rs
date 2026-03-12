//! TaskTimer - 任务定时器
//!
//! 用于记录定时任务的最后执行时间
//! 文件位置: tmp/lockid/{name}.time

use std::path::PathBuf;
use std::time::Instant;

use crate::project_path::ProjectPath;

pub struct TaskTimer {
    time_path: PathBuf,
}

impl TaskTimer {
    pub fn new(name: &str) -> Self {
        let project_path = ProjectPath::find().unwrap_or_default();
        let time_path = project_path.join("tmp/lockid").join(format!("{}.time", name));
        
        Self { time_path }
    }

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

    pub fn get_last_time(&self) -> Option<u64> {
        if !self.time_path.exists() {
            return None;
        }
        
        let content = std::fs::read_to_string(&self.time_path).ok()?;
        content.trim().parse().ok()
    }

    pub fn time_path(&self) -> &PathBuf {
        &self.time_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_timer() {
        let timer = TaskTimer::new("test_timer");
        let _ = std::fs::remove_file(timer.time_path());
        
        assert!(timer.should_run(3600));
        
        timer.mark_done().unwrap();
        assert!(!timer.should_run(3600));
        
        let _ = std::fs::remove_file(timer.time_path());
    }
}
