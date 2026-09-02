# FrontMatter - YAML Frontmatter 解析器

## 第一性目的

**解决什么问题**：从 Markdown 文件中提取和操作 YAML frontmatter，支持任务文档管理。

**核心价值**：
- 解析功能：从 Markdown 提取 YAML frontmatter
- 更新功能：更新或添加 frontmatter 字段
- 渲染功能：将 frontmatter 渲染为完整 Markdown
- 任务支持：内置 tasks 数组支持

---

## 完成标准（可验证）

### ✅ 解析 frontmatter
```rust
let content = r#"---
kind: test
title: 测试任务
status: pending
---
# 内容
"#;
let fm = FrontMatter::parse(content).unwrap();
assert_eq!(fm.kind, "test");
```

### ✅ 更新字段
```rust
let content = r#"---
kind: micro
status: executing
---
# Content
"#;
let result = FrontMatter::update_field(content, "status", "completed").unwrap();
assert!(result.contains("status: completed"));
```

### ✅ 渲染 Markdown
```rust
let fm = FrontMatter::default();
let body = "## 管理员指示\n\n重构 xxx";
let result = fm.render(body);
assert!(result.starts_with("---\n"));
```

---

## 前置依赖

| 依赖 | 说明 |
|------|------|
| regex | 正则表达式匹配 |
| serde_yaml | YAML 序列化/反序列化 |
| serde_json | JSON 值处理 |

---

## 测试方案

### 主要逻辑测试

| 测试 | 输入 | 预期输出 | 验证方法 |
|------|------|----------|----------|
| 解析 frontmatter | content with YAML | FrontMatter 结构 | assert_eq!(fm.kind, "test") |
| 更新字段 | status -> completed | 更新后的字符串 | assert!(result.contains("completed")) |
| 添加字段 | title = "test" | 新增字段 | assert_eq!(fm.title, "test") |
| 渲染 | fm + body | 完整 Markdown | assert!(result.starts_with("---")) |

---

## 知识库

### 核心概念

| 概念 | 说明 |
|------|------|
| FrontMatter | frontmatter 结构体 |
| TaskInfo | 任务信息结构体 |
| extra | 扩展字段 HashMap |

### 方法签名

```rust
// 解析
pub fn from_file(path: &Path) -> Result<Self, String>
pub fn parse(content: &str) -> Result<Self, String>

// 序列化
pub fn to_yaml(&self) -> Result<String, String>
pub fn render(&self, body: &str) -> String

// 更新
pub fn update_field(content: &str, field: &str, new_value: &str) -> Result<String, String>

// 访问
pub fn get(&self, key: &str) -> Option<&Value>
```

### FrontMatter 字段

| 字段 | 类型 | 说明 |
|------|------|------|
| kind | String | 任务类型 |
| status | String | 任务状态 |
| check | String | 检查结果 |
| taskid | String | 任务ID |
| title | String | 标题 |
| menu | String | 菜单名 |
| tasks | Vec<TaskInfo> | 子任务列表 |
| rollback_count | i32 | 回滚计数 |
| extra | HashMap | 扩展字段 |

---

## 好坏示例

### ✅ 好示例：解析任务文档

```rust
let fm = FrontMatter::from_file(Path::new("docs/task/plan.md")).unwrap();
println!("任务类型: {}", fm.kind);
println!("任务状态: {}", fm.status);
```

### ✅ 好示例：更新任务状态

```rust
let content = fs::read_to_string("plan.md").unwrap();
let updated = FrontMatter::update_field(&content, "status", "completed").unwrap();
fs::write("plan.md", updated).unwrap();
```

### ❌ 坏示例：title 是显式字段，不应放入 extra

```rust
// 错误：title 是显式字段，放入 extra 会导致重复
let mut fm = FrontMatter::default();
fm.extra.insert("title".to_string(), Value::String("test".to_string())); // ❌
// 正确做法：
fm.title = "test".to_string(); // ✅
```

---

## 文件位置

- 类实现: `crates/base/src/frontmatter/frontmatter.rs`
- 技术文档: `crates/base/src/frontmatter/frontmatter.md`
- 模块入口: `crates/base/src/frontmatter/mod.rs`
