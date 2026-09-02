# TaskTimer - 任务定时器

## 第一性目的

**解决什么问题**：控制任务的执行频率，避免短时间内重复执行。

**核心价值**：
- 间隔控制：判断距离上次执行是否已超过指定时间
- 轻量实现：基于文件时间戳，无需数据库
- 跨平台：支持 Windows 和 Linux
- 简单易用：只需 `should_run()` + `mark_done()` 两个方法

---

## 与 TaskLock 的区别

| 特性 | TaskLock | TaskTimer |
|------|----------|-----------|
| **目的** | 防止同一任务多实例并发运行 | 控制任务执行频率/间隔 |
| **存储内容** | PID (进程ID) | Unix 时间戳 |
| **文件后缀** | `.lock` | `.time` |
| **典型场景** | "只能有一个实例在跑" | "每隔 N 秒执行一次" |
| **机制** | 挤出机制（新实例踢掉旧实例） | 时间间隔检查 |

---

## 完成标准（可验证）

### ✅ 创建任务定时器
```rust
let timer = TaskTimer::new("sync_task");
assert!(timer.time_path().to_string_lossy().contains("sync_task.time"));
```

### ✅ 检查执行间隔
```rust
let timer = TaskTimer::new("test_timer");
let _ = std::fs::remove_file(timer.time_path()); // 清理旧记录

// 首次执行，应该运行
assert!(timer.should_run(3600));

// 标记完成
timer.mark_done().unwrap();

// 刚完成，不应该运行
assert!(!timer.should_run(3600));
```

### ✅ 获取上次执行时间
```rust
let timer = TaskTimer::new("test_timer");
let _ = std::fs::remove_file(timer.time_path());

// 从未执行过
assert!(timer.get_last_time().is_none());

timer.mark_done().unwrap();
// 有执行记录
assert!(timer.get_last_time().is_some());
```

---

## 前置依赖

| 依赖 | 说明 |
|------|------|
| project_path | 项目路径查找 |

---

## 测试方案

### 主要逻辑测试

| 测试 | 输入 | 预期输出 | 验证方法 |
|------|------|----------|----------|
| 创建定时器 | new("task") | 时间路径正确 | assert!(path.contains("task.time")) |
| 首次检查 | should_run(3600) | 返回 true | assert!(timer.should_run(3600)) |
| 标记完成 | mark_done() | 文件创建 | assert!(time_path.exists()) |
| 间隔检查 | should_run(3600) | 返回 false | assert!(!timer.should_run(3600)) |

---

## 知识库

### 核心概念

| 概念 | 说明 |
|------|------|
| TaskTimer | 任务定时器结构体 |
| time_path | 时间文件路径 |
| interval_secs | 执行间隔（秒） |
| Unix 时间戳 | 存储在文件中的时间格式 |

### 工作原理

```
should_run(interval_secs)
    ↓
时间文件不存在？ → 返回 true（应该执行）
    ↓
读取文件中的时间戳
    ↓
当前时间 - 上次时间 >= 间隔？
    ↓
是 → 返回 true（应该执行）
否 → 返回 false（跳过执行）

mark_done()
    ↓
获取当前 Unix 时间戳
    ↓
写入时间文件
```

### 方法签名

```rust
// 创建
pub fn new(name: &str) -> Self
pub fn with_path(relative_path: &str) -> Self

// 检查
pub fn should_run(&self, interval_secs: u64) -> bool

// 标记
pub fn mark_done(&self) -> Result<(), String>

// 信息
pub fn get_last_time(&self) -> Option<u64>
pub fn time_path(&self) -> &PathBuf
```

---

## 好坏示例

### ✅ 好示例：定时同步任务

```rust
let timer = TaskTimer::new("inventory_sync");

// 每小时同步一次
if timer.should_run(3600) {
    match sync_inventory() {
        Ok(_) => {
            timer.mark_done().expect("标记完成失败");
            println!("同步完成");
        }
        Err(e) => {
            eprintln!("同步失败: {}", e);
            // 不标记完成，下次继续尝试
        }
    }
}
```

### ✅ 好示例：使用自定义路径

```rust
let timer = TaskTimer::with_path("tmp/localid/custom_task.time");

if timer.should_run(7200) {
    do_task();
    timer.mark_done()?;
}
```

### ❌ 坏示例：忘记标记完成

```rust
let timer = TaskTimer::new("task");

if timer.should_run(3600) {
    do_task();
    // 忘记调用 mark_done()
    // 导致每次检查都会执行
}
```

### ❌ 坏示例：错误处理不当

```rust
let timer = TaskTimer::new("task");

if timer.should_run(3600) {
    do_task();
    timer.mark_done()?;  // 失败时应该处理，而不是忽略
}
```

---

## 实际使用案例

### buysell_workflow.rs

```rust
// 每 2 小时同步库存到策略表
let inventory_sync_timer = TaskTimer::with_path("tmp/localid/buysell_insert_store.time");

if inventory_sync_timer.should_run(7200) {
    match holding.sync_from_inventory("buysell", "BuySell 同步库存") {
        Ok(count) => {
            let _ = inventory_sync_timer.mark_done();
        }
        Err(e) => {
            logger.error(&format!("同步失败: {}", e));
        }
    }
}
```

### strategy_workflow.rs

```rust
// 每小时同步库存到策略表
let inventory_sync_timer = TaskTimer::new("sync_from_inventory");

if inventory_sync_timer.should_run(3600) {
    match holding.sync_from_inventory("strategy_cent", "每小时同步库存") {
        Ok(count) => {
            let _ = inventory_sync_timer.mark_done();
        }
        Err(e) => {
            logger.error(&format!("同步失败: {}", e));
        }
    }
}
```

---

## 文件位置

- 类实现：`crates/base/src/task_lock/task_timer.rs`
- 技术文档：`crates/base/src/task_lock/task_timer.md`
- 模块入口：`crates/base/src/task_lock/mod.rs`
