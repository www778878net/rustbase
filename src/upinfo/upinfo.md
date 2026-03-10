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

### 基础功能测试
- [ ] 创建 UpInfo 正确
- [ ] Guest 账号正确
- [ ] ID 生成正确

### 数据操作测试
- [ ] 设置 jsdata 正确
- [ ] 获取数据正确
- [ ] 列检查正确

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