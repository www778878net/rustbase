# TaskLock - 任务锁

## 第一性目的

**解决什么问题**：防止同一任务并发执行，支持挤出机制（优雅退出）。

**核心价值**：
- 单实例保证：同一时间只有一个任务实例运行
- 挤出机制：新实例启动时，旧实例自动退出
- 跨平台：支持 Windows 和 Linux
- 自动清理：Drop 时自动释放锁

---

## 完成标准（可验证）

### ✅ 创建任务锁
```rust
let lock = TaskLock::new("my_task");
assert!(lock.lock_path().to_string_lossy().contains("my_task.lock"));
```

### ✅ 获取和释放锁
```rust
let lock = TaskLock::new("test_task");
let _ = std::fs::remove_file(lock.lock_path()); // 清理旧锁
lock.try_acquire().unwrap();
assert!(lock.lock_path().exists());
lock.release().unwrap();
assert!(!lock.lock_path().exists());
```

### ✅ 检测被挤出
```rust
let lock = TaskLock::new("test_task");
lock.try_acquire().unwrap();
// 未被挤出
assert!(!lock.check_kicked());
// 模拟被挤出
std::fs::write(lock.lock_path(), "99999").unwrap();
assert!(lock.check_kicked());
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
| 创建锁 | new("task") | 锁路径正确 | assert!(path.contains("task.lock")) |
| 获取锁 | try_acquire() | 锁文件创建 | assert!(lock_path.exists()) |
| 释放锁 | release() | 锁文件删除 | assert!(!lock_path.exists()) |
| 挤出检测 | check_kicked() | 返回 bool | assert!(lock.check_kicked()) |

---

## 知识库

### 核心概念

| 概念 | 说明 |
|------|------|
| TaskLock | 任务锁结构体 |
| lock_path | 锁文件路径 |
| running | 运行标志 |
| 挤出机制 | 新实例启动，旧实例自动退出 |

### 工作原理

```
新实例启动
    ↓
读取锁文件中的 PID
    ↓
PID 相同 → 已持有锁
PID 不同 → 清空锁文件，写入自己的 PID
    ↓
旧实例检测到 PID 变化
    ↓
旧实例优雅退出
```

### 方法签名

```rust
// 创建
pub fn new(name: &str) -> Self
pub fn with_path(lock_path: PathBuf) -> Self

// 获取/释放
pub fn try_acquire(&self) -> Result<bool, String>
pub fn try_acquire_with_log<F>(&self, log_fn: F) -> Result<bool, String>
pub fn release(&self) -> Result<(), String>

// 检测
pub fn check_kicked(&self) -> bool
pub fn running_flag(&self) -> Arc<AtomicBool>

// 信息
pub fn lock_path(&self) -> &PathBuf
pub fn pid(&self) -> u32
```

---

## 好坏示例

### ✅ 好示例：在循环中检测挤出

```rust
let lock = TaskLock::new("background_task");
lock.try_acquire().unwrap();

let running = lock.running_flag();
while running.load(Ordering::SeqCst) {
    // 定期检测是否被挤出
    if lock.check_kicked() {
        println!("被新实例挤出，退出...");
        break;
    }
    // 执行任务
    do_work();
    thread::sleep(Duration::from_secs(1));
}
```

### ✅ 好示例：使用 Drop 自动释放

```rust
{
    let lock = TaskLock::new("task");
    lock.try_acquire().unwrap();
    // 执行任务...
} // 自动调用 Drop，释放锁
```

### ❌ 坏示例：未检测挤出

```rust
let lock = TaskLock::new("task");
lock.try_acquire().unwrap();

loop {
    do_work();
    // 缺少 check_kicked() 检测
    // 新实例启动后，旧实例不会退出
}
```

---

## 文件位置

- 类实现: `crates/base/src/task_lock/task_lock.rs`
- 技术文档: `crates/base/src/task_lock/task_lock.md`
- 模块入口: `crates/base/src/task_lock/mod.rs`
