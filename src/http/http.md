# HttpHelper - HTTP 请求工具

## 第一性目的

**解决什么问题**：提供统一的 HTTP 请求处理，支持代理配置、重试机制、超时处理。

**核心价值**：
- 统一接口：GET/POST 请求统一封装
- 代理支持：支持 SOCKS5 代理
- 重试机制：失败自动重试
- 超时控制：可配置读写超时

---

## 完成标准（可验证）

### ✅ GET 请求成功
```rust
let result = HttpHelper::get(
    "https://httpbin.org/get",
    None,
    None,
    false,
    None,
    30,
    0
);
assert_eq!(result.res, 0);
```

### ✅ POST JSON 成功
```rust
let json = serde_json::json!({"key": "value"});
let result = HttpHelper::post(
    "https://httpbin.org/post",
    None,
    Some(&json),
    None,
    false,
    None,
    30,
    0
);
assert_eq!(result.res, 0);
```

### ✅ 超时处理
```rust
let result = HttpHelper::get(
    "https://httpbin.org/delay/10",
    None,
    None,
    false,
    None,
    2,  // 2秒超时
    0
);
assert_eq!(result.res, -1);
```

---

## 前置依赖

| 依赖 | 说明 |
|------|------|
| ureq | HTTP 客户端库 |
| serde_json | JSON 处理 |

---

## 测试方案

### 主要逻辑测试

| 测试 | 输入 | 预期输出 | 验证方法 |
|------|------|----------|----------|
| 简单 GET | httpbin.org/get | res=0 | assert_eq!(result.res, 0) |
| 带参数 GET | params=[("foo", "bar")] | res=0 | assert_eq!(result.res, 0) |
| POST JSON | json={"key": "value"} | res=0 | assert_eq!(result.res, 0) |
| POST 表单 | form=[("user", "test")] | res=0 | assert_eq!(result.res, 0) |
| 超时测试 | delay/10, timeout=2 | res=-1 | assert_eq!(result.res, -1) |
| 重试测试 | status/500, retry=2 | res=-1 | assert_eq!(result.res, -1) |

---

## 知识库

### 核心概念

| 概念 | 说明 |
|------|------|
| HttpResponse | 响应结果结构体 |
| ResponseData | 响应数据详情 |
| HTTP_PROXY | 环境变量，配置代理地址（可选） |

### 代理配置

代理通过环境变量 `HTTP_PROXY` 或 `http_proxy` 配置：

```bash
# 设置代理
export HTTP_PROXY=socks5://127.0.0.1:10808

# 或
export http_proxy=http://proxy.example.com:8080
```

默认不使用代理，只有 `use_proxy=true` 且设置了环境变量时才启用。

### 方法签名

```rust
// GET 请求
pub fn get(
    url: &str,
    headers: Option<&[(&str, &str)]>,
    params: Option<&[(&str, &str)]>,
    use_proxy: bool,
    proxy: Option<&str>,
    timeout: u64,
    max_retries: u32,
) -> HttpResponse

// POST 请求
pub fn post(
    url: &str,
    data: Option<&[(&str, &str)]>,
    json_data: Option<&Value>,
    headers: Option<&[(&str, &str)]>,
    use_proxy: bool,
    proxy: Option<&str>,
    timeout: u64,
    max_retries: u32,
) -> HttpResponse
```

---

## 好坏示例

### ✅ 好示例：简单 GET

```rust
let result = HttpHelper::get(
    "https://httpbin.org/get",
    None,
    None,
    false,
    None,
    30,
    0
);
if result.res == 0 {
    println!("响应: {:?}", result.data);
}
```

### ✅ 好示例：带重试的请求

```rust
let result = HttpHelper::get(
    "https://api.example.com/data",
    Some(&[("Authorization", "Bearer token")]),
    None,
    true,  // 使用代理
    None,  // 默认代理
    30,
    3      // 重试3次
);
```

### ❌ 坏示例：无超时设置

```rust
// 错误：无超时可能导致永久阻塞
let result = HttpHelper::get(url, None, None, false, None, 0, 0);
```

---

## 文件位置

- 类实现: `crates/base/src/http/http.rs`
- 技术文档: `crates/base/src/http/http.md`
- 测试文件: `crates/base/src/http/test_http.rs`
- 模块入口: `crates/base/src/http/mod.rs`