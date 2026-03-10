# Config - 配置管理

## 第一性目的

**解决什么问题**：提供统一的配置管理，支持 JSON 文件加载、单例模式、表配置集成。

**核心价值**：
- 单例模式：全局唯一配置实例
- JSON 加载：支持从文件加载配置
- 类型安全：强类型配置访问
- 表配置集成：与 TableConfigManager 集成

---

## 完成标准（可验证）

### ✅ 创建配置实例
```rust
let config = Config::new_instance();
assert!(config.get("notexist").is_none());
```

### ✅ 设置和获取配置
```rust
let mut config = Config::new_instance();
config.set("key1", json!("value1"));
assert_eq!(config.get_string("key1"), Some("value1".to_string()));
```

### ✅ 单例模式
```rust
let config1 = Config::get_instance();
let config2 = Config::get_instance();
assert!(Arc::ptr_eq(&config1, &config2));
```

---

## 前置依赖

| 依赖 | 说明 |
|------|------|
| serde_json | JSON 处理 |
| table_config | 表配置模块 |

---

## 测试方案

### 主要逻辑测试

| 测试 | 输入 | 预期输出 | 验证方法 |
|------|------|----------|----------|
| 创建实例 | Config::new_instance() | 实例创建成功 | assert!(config.get("notexist").is_none()) |
| 设置获取 | set("key1", "value1") | get_string = "value1" | assert_eq!(config.get_string("key1"), Some("value1")) |
| 单例模式 | get_instance() x2 | 同一实例 | assert!(Arc::ptr_eq(&config1, &config2)) |

---

## 知识库

### 核心概念

| 概念 | 说明 |
|------|------|
| Config | 配置结构体 |
| ConfigError | 配置错误枚举 |
| TableConfigManager | 表配置管理器 |

### 方法签名

```rust
// 创建新实例（非单例）
pub fn new_instance() -> Self

// 获取全局实例（单例）
pub fn get_instance() -> Arc<Config>

// 获取配置项
pub fn get(&self, key: &str) -> Option<&Value>
pub fn get_string(&self, key: &str) -> Option<String>
pub fn get_int(&self, key: &str) -> Option<i64>
pub fn get_bool(&self, key: &str) -> Option<bool>

// 设置配置项
pub fn set(&mut self, key: &str, value: Value)

// 表配置
pub fn get_table(&self, table_name: &str) -> Option<&TableSet>
pub fn table_names(&self) -> Vec<&String>
```

---

## 好坏示例

### ✅ 好示例：使用单例获取配置

```rust
let config = Config::get_instance();
if let Some(db_url) = config.get_string("database_url") {
    // 使用配置
}
```

### ✅ 好示例：初始化自定义配置文件

```rust
let mut config = Config::new_instance();
config.init(Some("config/production.json")).unwrap();
```

### ❌ 坏示例：未初始化就访问配置

```rust
let config = Config::new_instance();
// 未调用 init()，配置可能为空
let db_url = config.get_string("database_url").expect("需要配置");
```

---

## 文件位置

- 类实现: `crates/base/src/config/config.rs`
- 技术文档: `crates/base/src/config/config.md`
- 模块入口: `crates/base/src/config/mod.rs`
