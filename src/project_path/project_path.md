# ProjectPath - 项目路径查找工具

## 第一性目的

**解决什么问题**：提供项目根目录查找、默认路径生成、配置加载等能力。

**核心价值**：
- 自动查找：从当前目录向上查找 docs 或 .claude 目录
- 路径生成：提供常用路径快捷方法
- 配置加载：支持 INI 和简单文本配置

---

## 完成标准（可验证）

### ✅ 查找项目根目录
```rust
let project = ProjectPath::find().expect("找不到项目根目录");
assert!(project.root().join("docs").exists() || project.root().join(".claude").exists());
```

### ✅ 获取本地数据库路径
```rust
let project = ProjectPath::find().expect("找不到项目根目录");
let db_path = project.local_db();
assert!(db_path.ends_with("docs/config/local.db"));
```

### ✅ 加载 INI 配置
```rust
let project = ProjectPath::find().expect("找不到项目根目录");
let config = project.load_ini_config();
```

---

## 前置依赖

| 依赖 | 说明 |
|------|------|
| 无外部依赖 | 仅使用标准库 |

---

## 测试方案

### 主要逻辑测试

| 测试 | 输入 | 预期输出 | 验证方法 |
|------|------|----------|----------|
| 查找根目录 | find() | Ok(ProjectPath) | root 包含 docs 或 .claude |
| 本地数据库路径 | local_db() | docs/config/local.db | 路径结尾正确 |
| 配置目录 | docs_config() | docs/config | 路径结尾正确 |
| 环境检测 | environment() | Environment | 根据APP_ENV返回正确环境 |
| INI配置解析 | parse_ini_content() | HashMap | 正确解析section和key-value |
| 从指定路径查找 | find_from(path) | Ok(ProjectPath) | 从指定路径向上查找 |

### 其它测试（边界、异常等）

| 测试 | 输入 | 预期输出 | 验证方法 |
|------|------|----------|----------|
| 不存在的路径查找 | find_from(无效路径) | Err | 返回错误信息 |
| 环境变量覆盖 | read_text_config() | 环境变量值 | 环境变量优先于文件 |
| Worker名称获取 | worker_name() | Optional<String> | 环境变量或文件读取 |
| 路径拼接 | join("relative") | 拼接后路径 | 路径正确拼接 |

---

## 知识库

### 核心概念

| 概念 | 说明 |
|------|------|
| root | 项目根目录 |
| environment | 运行环境（Development/Test/Production） |
| docs() | docs 目录路径 |
| docs_config() | docs/config 目录路径 |
| local_db() | 本地数据库路径 |

### 方法签名

```rust
// 查找项目根目录
pub fn find() -> Result<Self, String>

// 从指定路径开始查找
pub fn find_from(start: &PathBuf) -> Result<Self, String>

// 获取各种路径
pub fn root(&self) -> &PathBuf
pub fn docs(&self) -> PathBuf
pub fn docs_config(&self) -> PathBuf
pub fn local_db(&self) -> PathBuf
pub fn logs(&self) -> PathBuf

// 配置加载
pub fn load_ini_config(&self) -> Result<HashMap<String, HashMap<String, String>>, String>
pub fn read_text_config(&self, filename: &str, env_key: &str) -> Option<String>
```

---

## 好坏示例

### ✅ 好示例：查找项目根目录

```rust
let project = ProjectPath::find().expect("找不到项目根目录");
println!("项目根目录: {:?}", project.root());
```

### ✅ 好示例：获取配置文件路径

```rust
let project = ProjectPath::find().expect("找不到项目根目录");
let config_path = project.config_file("worker.txt");
```

### ❌ 坏示例：假设固定路径

```rust
// 错误：硬编码路径
let db_path = "/workspace/project/docs/config/local.db";
```

---

## 文件位置

- 类实现: `crates/base/src/project_path/project_path.rs`
- 技术文档: `crates/base/src/project_path/project_path.md`
- 测试文件: `crates/base/src/project_path/test_project_path.rs`
- 模块入口: `crates/base/src/project_path/mod.rs`