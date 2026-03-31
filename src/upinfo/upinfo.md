# UpInfo 请求上下文文档

## 管理员指示
- 本文档描述 UpInfo 模块的设计和实现
- 完整移植自 koa78-base78/UpInfo.ts
- 禁止把代码写在 mod.rs, lib.rs，应该在子模块中实现

## 第一性目的
- API 请求上下文
- 支持 jsdata (JSON) 和 bytedata (二进制) 双格式
- 统一请求/响应结构

## 完成标准
- 创建 UpInfo 正确
- 数据获取正确
- 响应构造正确

## 前置依赖
- serde 库
- chrono 库
- uuid 库
- base64 库

## 业务逻辑

### 核心字段
- `sid`: 会话 ID
- `cid`: 公司 ID (数据隔离)
- `uid`: 用户 ID
- `jsdata`: JSON 格式数据
- `bytedata`: 二进制格式数据

### 数据获取参数
- `getstart`: 起始位置
- `getnumber`: 获取数量
- `order`: 排序
- `cols`: 列选择

### 响应结构
- `res`: 结果码，0=成功
- `errmsg`: 错误信息
- `jsdata`: JSON 数据
- `bytedata`: 二进制数据

## 测试方案

### 主要逻辑测试

| 测试 | 输入 | 预期输出 | 验证方法 |
|------|------|----------|----------|
| 创建 UpInfo | new() | 默认值正确 | getstart=0, getnumber=15 |
| Guest 账号 | get_guest() | sid 包含 GUEST | assert!(sid.contains("GUEST")) |
| 默认账号 | default_upinfo() | cid=default | assert_eq!(cid, "default") |
| ID 生成 | new_id() | UUID格式 | len==36, 唯一 |
| 时间戳ID | new_id_ts() | 时间戳格式 | len>10 |
| 设置 API | with_api() | 路径正确 | apisys/apimicro/apiobj |
| 设置 jsdata | with_jsdata() | jsdata 有值 | get_raw_data() 返回值 |

### 其它测试（边界、异常等）

| 测试 | 输入 | 预期输出 | 验证方法 |
|------|------|----------|----------|
| 获取数据无数据 | get_data() | Err(NoData) | assert!(matches!(result, Err(NoData))) |
| 获取数据JSON | get_data() | Ok(T) | JSON 解析成功 |
| 列检查全部 | check_cols() | checkcolsallok | cols=["all"] |
| 列检查特定 | check_cols() | checkcolsallok | cols 在允许列表 |
| 列检查无效 | check_cols() | 返回无效列名 | 返回无效列 |
| 排序检查 | in_order() | bool | 排序字段是否有效 |
| 响应成功 | Response::success_json() | res=0 | 有 back 数据 |
| 响应失败 | Response::fail() | res<0 | 有 errmsg |

## 知识库

### 创建 UpInfo
```rust
let up = UpInfo::new();
let up = UpInfo::get_guest();
let up = UpInfo::default_upinfo();
```

### 设置数据
```rust
let up = UpInfo::new()
    .with_api("api", "basic", "test")
    .with_jsdata(r#"{"key": "value"}"#);
```

### 创建响应
```rust
let resp = Response::success_json(&data);
let resp = Response::fail("错误信息", -1);
```

## 好坏示例

### 好示例
- 使用 with_api() 链式设置
- 使用 Response 工厂方法

### 坏示例
- 手动设置所有字段
- 忽略错误检查