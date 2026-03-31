# 管理员指示禁止删除必须保留，且为本目录最高指示

## 第一性目的
- 工作流日志统一管理，禁止直接使用标准 logging 模块
- 全局日志固定写在 logs/project/ 下 
## 逻辑
- 开发环境：记录DEBUG及以上级别
- 
---

# MyLogger 日志库技术文档

## 第一性目的

### 为什么存在？
解决日志分散难以追踪的问题。所有工作流代码必须通过 MyLogger 记录日志，实现：
1. **统一入口**：禁止直接使用标准 logging，避免日志分散
2. **多目标写入**：一条日志同时写入 project.log、detail.log、工作流日志
3. **自动归档清理**：启动时归档旧日志，自动清理过期日志

### 解决什么问题？
| 问题         | 解决方案                |
| ------------ | ----------------------- |
| 日志分散各处 | 统一 MyLogger 入口      |
| 调试信息不足 | DETAIL 级别记录完整信息 |
| 生产日志过多 | 环境变量自动过滤级别    |
| 日志文件过大 | 自动归档 + 过期清理     |

## 完成标准

### 可验证的产品标准
| #   | 标准                                   | 验证方法                                                            |
| --- | -------------------------------------- | ------------------------------------------------------------------- |
| 1   | project.log 存在且包含 INFO 及以上日志 | `cat logs/project/project.log \| grep -E "INFO\|WARN\|ERROR"`       |
| 2   | detail.log 存在且包含所有级别日志      | `cat logs/project/detail.log \| grep "DETAIL"`                      |
| 3   | 工作流日志写入正确目录                 | `ls logs/{wfname}/{wfname}.log`                                     |
| 4   | 单例模式生效                           | 相同 caller_name 返回同一实例（Arc 指针相等）                       |
| 5   | 日志格式正确                           | `正则: \d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} - \w+ - \w+ - \[.*\] .+` |
| 6   | 过期日志已清理                         | 修改时间超过 retention_days 的日志文件不存在                        |

### 验证命令
```bash
# 运行测试
python3 crates/base/src/test_mylogger.py

# 验证日志文件
ls -la logs/project/project.log logs/project/detail.log
ls -la logs/{wfname}/{wfname}.log
```

## 前置依赖

### 必须满足的条件（可验证）
| 条件                            | 验证方法                       |
| ------------------------------- | ------------------------------ |
| Rust edition 2021+              | `cargo --version` 返回有效版本 |
| 项目根目录包含 src/ 或 .claude/ | `ls src/ \|\| ls .claude/`     |
| chrono 依赖已安装               | `grep chrono Cargo.toml`       |
| parking_lot 依赖已安装          | `grep parking_lot Cargo.toml`  |

## 业务逻辑

### 逻辑流转
```
get_logger(caller_name, wfname, retention_days)
    ↓
检查单例缓存 ──→ 命中 ──→ 返回已有实例
    ↓ 未命中
初始化全局日志（project.log, detail.log）
    ↓
创建工作流日志目录 logs/{wfname}/
    ↓
归档现有日志文件（重命名为 {name}_{timestamp}.log）
    ↓
清理过期日志（超过 retention_days）
    ↓
创建新实例并存入缓存
    ↓
返回 Arc<MyLogger>
```

### 数据流转
```
logger.info("message")
    ↓
格式化: [caller_name] message
    ↓
添加时间戳: 2026-02-28 21:30:00 - {wfname} - INFO - [caller_name] message
    ↓
┌─────────────────────────────────────────────────────┐
│ 并行写入（线程安全）                                   │
├─────────────────────────────────────────────────────┤
│ 1. project.log (INFO 及以上) ← Mutex 保护            │
│ 2. detail.log (所有级别) ← Mutex 保护                │
│ 3. {wfname}.log (根据级别过滤) ← Mutex 保护          │
│ 4. 控制台 (根据 console_level)                       │
└─────────────────────────────────────────────────────┘
```

### 状态流转
```
实例状态：
未初始化 → 初始化中 → 已初始化 → 缓存命中（复用）

日志文件状态：
不存在 → 创建 → 写入 → 归档（重命名） → 清理（删除）
```

### 并发控制
| 资源         | 锁类型 | 说明             |
| ------------ | ------ | ---------------- |
| project.log  | Mutex  | 全局文件写入锁   |
| detail.log   | Mutex  | 全局文件写入锁   |
| {wfname}.log | Mutex  | 实例级文件写入锁 |
| 实例缓存     | RwLock | 读多写少场景     |

## 测试方案

### 主要逻辑测试

| 测试 | 输入 | 预期输出 | 验证方法 |
|------|------|----------|----------|
| mylogger 宏 | mylogger!() | 实例创建成功 | assert!(logger.info("test") 正常执行) |
| 日志级别顺序 | LogLevel 比较 | Detail < Debug < Info < Warn < Error | assert!(Detail < Debug) |
| 级别转换 | from_i32(5/10/20/30/40) | 正确的 LogLevel | assert_eq!(from_i32(5), Detail) |
| 级别字符串 | as_str() | "DETAIL"/"DEBUG"/"INFO"/"WARN"/"ERROR" | assert_eq!(Detail.as_str(), "DETAIL") |
| 环境检测 | from_str("development") | Environment::Development | assert_eq!(env, Development) |
| 环境级别 | console_level()/file_level() | 正确级别 | Production=Info, Development=Debug |
| 单例模式 | get_logger("name") 两次 | 同一实例 | Arc::ptr_eq 返回 true |
| 环境切换 | set_environment() | 级别更新 | assert_eq!(logger.get_environment(), Development) |
| 错误格式化 | format_error(&error) | 包含异常信息 | assert!(formatted.contains("Exception")) |

### 其它测试（边界、异常等）

| 测试 | 输入 | 预期输出 | 验证方法 |
|------|------|----------|----------|
| 无效级别值 | from_i32(99) | 返回默认 Info | assert_eq!(from_i32(99), Info) |
| 错误链格式 | format_error() 包含 source | 完整错误链 | assert!(formatted.contains("Caused by")) |

## 知识库

### 日志级别定义
| 级别   | 值  | 控制台 | project.log | detail.log | 使用场景     |
| ------ | --- | ------ | ----------- | ---------- | ------------ |
| DETAIL | 5   | 否     | 否          | 是         | 完整调试信息 |
| DEBUG  | 10  | 是     | 否          | 是         | 开发调试     |
| INFO   | 20  | 是     | 是          | 是         | 正常业务     |
| WARN   | 30  | 是     | 是          | 是         | 警告         |
| ERROR  | 40  | 是     | 是          | 是         | 错误         |

### 环境配置
| APP_ENV     | 控制台级别 | 文件级别 |
| ----------- | ---------- | -------- |
| production  | INFO       | INFO     |
| development | DEBUG      | DETAIL   |
| test        | DEBUG      | DEBUG    |

### 文件结构
```
logs/
├── project/
│   ├── project.log           # 全局日志（INFO+）
│   ├── project_{timestamp}.log  # 归档日志
│   ├── detail.log            # 详细日志（全部）
│   └── detail2.log           # 上次运行备份
└── {wfname}/
    ├── {wfname}.log          # 当前日志
    └── {wfname}_{timestamp}.log  # 归档日志
```

### 公共接口
```rust
// 唯一推荐用法：mylogger!() 无参数宏
// 自动获取类名，retention=3（默认），禁止传参
let logger = mylogger!();

// 日志方法
fn detail(&self, message: &str)  // DETAIL 级别
fn debug(&self, message: &str)   // DEBUG 级别
fn info(&self, message: &str)    // INFO 级别
fn warn(&self, message: &str)    // WARN 级别
fn error(&self, message: &str)   // ERROR 级别
fn error_exc(&self, error: &dyn std::error::Error)  // 异常堆栈
```

## 好坏示例

### ✅ 好示例
```rust
// 好：mylogger!() 无参数，自动获取类名
let logger = mylogger!();
logger.info("开始处理");
logger.detail("详细调试信息");
```

```rust
// 好：记录异常堆栈
match risky_operation() {
    Ok(result) => logger.info(&format!("成功: {}", result)),
    Err(e) => logger.error_exc(&e),  // 完整堆栈
}
```

### ❌ 坏示例
```rust
// 坏：禁止手动传参，禁止带参数调用
let logger = MyLogger::new("Test", 3);  // ❌ 禁止
let logger = get_logger("Test", 3);     // ❌ 禁止
let logger = mylogger!(7);              // ❌ 禁止带参数
```

```rust
// 坏：只记录消息不记录堆栈
if let Err(e) = operation() {
    logger.error(&format!("错误: {}", e));  // 丢失堆栈信息
    // 应该用 logger.error_exc(&e)
}
```

```rust
// 坏：生产环境使用 DETAIL
logger.detail("生产环境不应该有");  // 污染代码，生产会过滤
// 生产代码应该用 info/warn/error
```

## 测试文件清单

| 文件             | 说明            |
| ---------------- | --------------- |
| mylogger.rs      | Rust 实现       |
| mylogger.log     | 真实运行日志    |
| mylogger.input   | 测试输入参数    |
| mylogger.check   | 测试输出验证    |
| test_mylogger.py | Python 测试套件 |