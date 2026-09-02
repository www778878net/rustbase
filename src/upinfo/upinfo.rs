//! UpInfo - 请求上下文
//!
//! 完整移植自 koa78-base78/UpInfo.ts
//! 支持 jsdata (JSON) 和 bytedata (二进制) 两种数据格式。
//!
//! ## 数据传输格式约定（bytedata / jsdata）
//! - **bytedata（protobuf）优先**：写入/返回主业务数据时优先使用 `bytedata`（protobuf 二进制，省字节、
//!   强 schema）。protobuf message 由各表 `colsImp` 生成（见 `other/logsvc/hooks/generateProtobufs.ts`）。
//! - **jsdata（JSON）仅作兼容/测试**：调试、单测、轻量场景可用 `jsdata`，生产主链路不依赖它。
//! - `svcbase` 作为框架无关核心**只持有 `bytedata: Vec<u8>` 原始字节，不做 protobuf 编解码**
//!   （保持 wasm32 可编译、无 prost 依赖）；protobuf 编解码由上层 `svccn`/`datastate` 负责。

use serde::{Deserialize, Serialize};
use chrono::Local;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

/// Master 实例 (全局静态)
static MASTER_INSTANCE: OnceLock<UpInfo> = OnceLock::new();

/// UpInfo - API 请求上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UpInfo {
    // ============ 数据获取非必填字段 ============
    /// 起始位置
    pub getstart: i32,
    /// 获取数量
    pub getnumber: i32,
    /// 排序（**已验证，业务只读**）：由 `Base78::check_request` 用 `validated_order` 校验 `ordern` 后写入。
    /// 拒绝任何上传输入（skip_deserializing），只能由代码写入，防止客户端直接指定排序绕过校验。
    #[serde(skip_deserializing)]
    pub order: String,
    /// 业务公司ID
    pub bcid: String,
    /// 业务主键
    pub mid: String,
    /// 请求ID
    pub upid: String,
    /// 类型
    #[serde(rename = "type")]
    pub type_: i32,

    // ============ 业务数据 (新格式) ============
    /// JSON 格式数据 (AI 友好)
    pub jsdata: Option<String>,
    /// 二进制格式数据 (省字节)
    pub bytedata: Option<Vec<u8>>,
    /// 查询过滤列（**已验证，业务只读**）：`get` 方法中作为 `WHERE col = ?` 条件的字段名清单。
    /// 其值由 `wherecolsn` 自带 KV 提供（方案 A，不再从 jsdata 取值）。`wherecols` 仅承载列名清单，
    /// 供审计/白名单口径；真正的 KV 值保留在 `wherecolsn` 中供 `get` 使用。
    /// 拒绝任何上传输入（skip_deserializing），只能由 `check_request` 从 `wherecolsn` 校验后写入，防止客户端直接指定条件列绕过白名单。
    #[serde(skip_deserializing)]
    pub wherecols: Vec<String>,
    /// 查询返回列（**已验证，业务只读**）：`get` 方法中 `SELECT` 要返回的列名（空则返回全部 `*`）。
    /// 拒绝任何上传输入（skip_deserializing），只能由 `check_request` 从 `getcolsn` 校验后写入。
    #[serde(skip_deserializing)]
    pub getcols: Vec<String>,

    // ============ 调试监控用 ============
    /// 调试模式
    pub debug: bool,
    /// PC ID
    pub pcid: String,
    /// PC 名称
    pub pcname: String,
    /// 来源
    pub source: String,
    /// API 版本号
    pub v: i32,
    /// 缓存
    pub cache: String,
    /// 外部回调的原始请求（解析不出内部协议 `EntryRequest` 时整体存放，保留原始格式，由 Controller 自行解释）。
    /// 如企微回调：body 为 XML 密文原始字节，query 为验签参数 msg_signature/timestamp/nonce/echostr。
    /// 内部协议请求（我们客户端发的）不设此字段，数据进 sid/jsdata/bytedata/wherecolsn 等语义字段。
    #[serde(default)]
    pub raw: Option<RawRequest>,

    /// SSE 流式事件接收器（svccn 通用能力）。
    ///
    /// Controller 设置 `backtype = "sse"` 并把事件流 receiver 塞进此字段时，
    /// `svccn` 的 HTTP 适配层返回 `Sse<ReceiverStream>`（而非 JSON）。
    /// 仅内存传递；`#[serde(skip)]` 不参与请求解析/响应序列化，
    /// 用 `Arc<Mutex<Option<Receiver>>>` 以满足 derive 的 `Clone + Default`（Receiver 本身不可 Clone/Default）。
    #[serde(skip)]
    pub sse_rx: Option<Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::Receiver<serde_json::Value>>>>>,

    // ============ 自动获取或服务器生成 ============
    /// IP 地址
    pub ip: String,
    /// 方法路径
    pub method: String,

    /// 系统 (多个微服务组合)
    pub apisys: String,
    /// 微服务
    pub apimicro: String,
    /// 对象 (表/类)
    pub apiobj: String,
    /// 函数 (方法)
    pub apifun: String,
    /// 请求时间
    pub uptime: String,
    /// 操作人
    pub upby: String,
    /// 错误信息
    pub errmessage: String,

    // ============ 上传临时存 验证后再用 ============
    /// 公司ID (待验证)
    pub cidn: String,

    // ============ 上传原始值（未验证，仅入口/框架内部流转，业务不可读） ============
    /// @unverified 上传原始 WHERE 条件（列名+值的 KV 对，方案 A：自带值，不再从 jsdata 取值）。
    /// 每个元素是一组 `col -> value`，多个元素之间为 AND 关系（同元素内多字段亦为 AND）。
    /// 由 `Base78::check_request` 用 `cols_imp_set` 白名单逐 key 校验后，把列名写入已验证的
    /// `wherecols`（列名清单）。值保留在本字段 KV 中，供 `get` 直接遍历取「列名 + 值」拼 `AND col = ?`。
    /// 业务只读 `wherecols`，绝不可直接读本字段。
    pub wherecolsn: Vec<HashMap<String, serde_json::Value>>,
    /// @unverified 上传原始 SELECT 列名。由 `Base78::check_request` 用 `cols_imp_set` 白名单校验后写入 `getcols`。
    /// 业务只读 `getcols`，绝不可直接读本字段。
    pub getcolsn: Vec<String>,
    /// @unverified 上传原始排序字段。由 `Base78::check_request` 用 `validated_order` 校验（非法回退 `id DESC`）后写入 `order`。
    /// 业务只读 `order`，绝不可直接读本字段。
    pub ordern: String,

    // ============ 需数据库读取验证 ============
    /// 会话 ID
    pub sid: String,
    /// 公司 ID (数据隔离，**已验证，业务只读**)：由 SID 会话反查填充，拒绝任何上传输入（skip_deserializing），防止越权指定账套。
    #[serde(skip_deserializing)]
    pub cid: String,
    /// 用户 ID (数据隔离，**已验证，业务只读**)：由 SID 会话反查填充，拒绝任何上传输入（skip_deserializing）。
    #[serde(skip_deserializing)]
    pub uid: String,
    /// 公司名
    pub coname: String,
    /// 用户名
    pub uname: String,

    /// 密码
    pub pwd: String,
    /// 微信
    pub weixin: String,
    /// CEO ID
    pub idceo: String,
    /// 真实姓名
    pub truename: String,
    /// 手机号
    pub mobile: String,

    // ============ 返回用 ============
    /// 结果码: 0 成功, 负数失败
    pub res: i32,
    /// 错误信息
    pub errmsg: String,
    /// 返回类型
    pub backtype: String,

    /// JSONP
    pub jsonp: bool,
    /// Base64
    pub base64: bool,
    /// JSON
    pub json: bool,

    // ============ 弃用下版删除 ============
    /// JSON Base64 (弃用)
    pub jsonbase64: bool,
}

/// 外部回调的原始请求（`UpInfo.raw` 承载）。
///
/// 当 4 层路由入口无法把 body 解析成内部协议 `EntryRequest` 时（如企微回调的 XML 密文、验签空 body），
/// 把**原始输入整体**存入本结构，保留原始格式，由 Controller 按需解释（`body` 需转文本时自行 `String::from_utf8`）。
/// 统一用原始字节（`Vec<u8>`）承载，不区分文本/二进制——"BYTE 与文本同源，都是原始字节"。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawRequest {
    /// 原始 Content-Type（如 application/xml、text/plain）
    #[serde(default)]
    pub content_type: String,
    /// 原始 body 字节（文本也是 UTF-8 字节，统一存原始字节）
    #[serde(default)]
    pub body: Vec<u8>,
    /// query 参数（GET 验签的 msg_signature/timestamp/nonce/echostr 等）
    #[serde(default)]
    pub query: HashMap<String, String>,
}

impl UpInfo {
    /// 创建新 UpInfo
    pub fn new() -> Self {
        Self {
            getstart: 0,
            getnumber: 15,
            order: "id desc".to_string(),
            bcid: String::new(),
            mid: Self::new_id(),
            upid: Self::new_id(),
            type_: 0,

            jsdata: None,
            bytedata: None,
            wherecols: vec![],
            getcols: vec![],

            debug: false,
            pcid: String::new(),
            pcname: String::new(),
            source: "no".to_string(),
            v: 24,
            cache: String::new(),
            raw: None,
            sse_rx: None,

            ip: String::new(),
            method: String::new(),

            apisys: String::new(),
            apimicro: String::new(),
            apiobj: String::new(),
            apifun: String::new(),
            uptime: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            upby: String::new(),
            errmessage: String::new(),

            cidn: String::new(),

            wherecolsn: vec![],
            getcolsn: vec![],
            ordern: String::new(),

            sid: String::new(),
            cid: String::new(),
            uid: String::new(),
            coname: "测试帐套".to_string(),
            uname: "guest".to_string(),

            pwd: String::new(),
            weixin: String::new(),
            idceo: String::new(),
            truename: String::new(),
            mobile: String::new(),

            res: 0,
            errmsg: String::new(),
            backtype: "json".to_string(),

            jsonp: false,
            base64: false,
            json: true,
            jsonbase64: false,
        }
    }

    /// 创建 Guest UpInfo
    pub fn get_guest() -> Self {
        Self {
            sid: "GUEST888-8888-8888-8888-GUEST88GUEST".to_string(),
            cid: "GUEST000-8888-8888-8888-GUEST00GUEST".to_string(),
            bcid: "d4856531-e9d3-20f3-4c22-fe3c65fb009c".to_string(),
            mid: Self::new_id(),
            uname: "guest".to_string(),
            getstart: 0,
            ip: "127.0.0.1".to_string(),
            ..Self::new()
        }
    }

    /// 创建默认 UpInfo (用于测试)
    pub fn default_upinfo() -> Self {
        Self {
            sid: "GUEST888-8888-8888-8888-GUEST88GUEST".to_string(),
            cid: "default".to_string(),
            uid: "test".to_string(),
            uname: "tester".to_string(),
            apisys: "api".to_string(),
            apimicro: "basic".to_string(),
            apiobj: "test".to_string(),
            ..Self::new()
        }
    }

    /// 生成新 ID (业务主键) - UUID 格式
    pub fn new_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// 生成新 ID (时间戳格式)
    pub fn new_id_ts() -> String {
        let ts = Local::now().format("%Y%m%d%H%M%S").to_string();
        let suffix = uuid::Uuid::new_v4().to_string()[..6].to_string();
        format!("{}{}", ts, suffix)
    }

    /// 设置 API 路径
    pub fn with_api(mut self, apisys: &str, apimicro: &str, apiobj: &str) -> Self {
        self.apisys = apisys.to_string();
        self.apimicro = apimicro.to_string();
        self.apiobj = apiobj.to_string();
        self
    }

    /// 设置 jsdata
    pub fn with_jsdata(mut self, json: &str) -> Self {
        self.jsdata = Some(json.to_string());
        self
    }

    /// 设置 bytedata
    pub fn with_bytedata(mut self, data: Vec<u8>) -> Self {
        self.bytedata = Some(data);
        self
    }

    /// 获取业务数据 (优先 jsdata)
    pub fn get_data<T: for<'de> Deserialize<'de>>(&self) -> Result<T, UpInfoError> {
        if let Some(ref json) = self.jsdata {
            serde_json::from_str(json).map_err(UpInfoError::JsonError)
        } else if let Some(ref bytes) = self.bytedata {
            let json: String = serde_json::from_slice(bytes)
                .map_err(|e| UpInfoError::ByteError(e.to_string()))?;
            serde_json::from_str(&json).map_err(UpInfoError::JsonError)
        } else {
            Err(UpInfoError::NoData)
        }
    }

    /// 解析带前缀的数据
    /// 前8位类型标记: "00000000" = bytedata(protobuf), "00000001" = jsdata(json)
    /// 返回解析后的 JSON Value
    pub fn parse_prefixed_data(&self) -> Result<serde_json::Value, UpInfoError> {
        if let Some(ref data) = self.jsdata {
            if data.len() < 8 {
                return Err(UpInfoError::FormatError("数据长度不足8位".to_string()));
            }
            
            let prefix = &data[..8];
            let content = &data[8..];
            
            let first_char = prefix.chars().next().unwrap_or(' ');
            match first_char {
                '0' => {
                    Err(UpInfoError::FormatError("bytedata格式请使用bytedata字段".to_string()))
                }
                '1' => {
                    serde_json::from_str(content).map_err(UpInfoError::JsonError)
                }
                _ => {
                    serde_json::from_str(data).map_err(UpInfoError::JsonError)
                }
            }
        } else if let Some(ref bytes) = self.bytedata {
            if bytes.len() < 8 {
                return Err(UpInfoError::FormatError("数据长度不足8字节".to_string()));
            }
            
            let first_byte = bytes[0];
            let content = &bytes[8..];
            
            match first_byte {
                0 => {
                    let json: String = serde_json::from_slice(content)
                        .map_err(|e| UpInfoError::ByteError(e.to_string()))?;
                    serde_json::from_str(&json).map_err(UpInfoError::JsonError)
                }
                1 => {
                    let json_str = String::from_utf8_lossy(content);
                    serde_json::from_str(&json_str).map_err(UpInfoError::JsonError)
                }
                _ => {
                    let json: String = serde_json::from_slice(bytes)
                        .map_err(|e| UpInfoError::ByteError(e.to_string()))?;
                    serde_json::from_str(&json).map_err(UpInfoError::JsonError)
                }
            }
        } else {
            Err(UpInfoError::NoData)
        }
    }

    /// 获取原始 JSON 字符串
    pub fn get_raw_data(&self) -> Option<&str> {
        self.jsdata.as_deref()
    }

    /// 检查排序是否有效
    pub fn in_order(&self, cols: &[&str]) -> bool {
        let orders: Vec<&str> = self.order.split(',').collect();

        for o in orders {
            let order = o.trim();
            let order_name = if order.ends_with(" desc") {
                &order[..order.len() - 5]
            } else {
                order
            };

            if order_name == "id" || order_name == "id" || order_name == "uptime" || order_name == "upby" {
                continue;
            }

            if !cols.contains(&order_name) {
                return false;
            }
        }

        true
    }

    /// 克隆
    pub fn clone_upinfo(&self) -> Self {
        self.clone()
    }

    /// 解码 Base64
    pub fn decode_base64(&self, encoded: &str) -> String {
        let normalized = encoded
            .replace('*', "+")
            .replace('-', "/")
            .replace('.', "=");
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD
            .decode(&normalized)
            .map(|v| String::from_utf8_lossy(&v).to_string())
            .unwrap_or_default()
    }

    // ============ 静态方法 ============

    /// 设置 Master 实例
    pub fn set_master(up: UpInfo) {
        let _ = MASTER_INSTANCE.set(up);
    }

    /// 获取 Master 实例
    pub fn get_master() -> UpInfo {
        let up = MASTER_INSTANCE
            .get()
            .cloned()
            .unwrap_or_else(Self::get_guest);
        up
    }
}

impl Default for UpInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// UpInfo 错误
#[derive(Debug)]
pub enum UpInfoError {
    NoData,
    JsonError(serde_json::Error),
    ByteError(String),
    FormatError(String),
}

impl std::fmt::Display for UpInfoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpInfoError::NoData => write!(f, "无业务数据"),
            UpInfoError::JsonError(e) => write!(f, "JSON 解析失败: {}", e),
            UpInfoError::ByteError(e) => write!(f, "字节数据解析失败: {}", e),
            UpInfoError::FormatError(e) => write!(f, "格式错误: {}", e),
        }
    }
}

impl std::error::Error for UpInfoError {}

/// API 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// 结果码: 0 成功, 负数失败
    pub res: i32,
    /// 错误信息
    pub errmsg: String,
    /// 返回类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// 实际数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub back: Option<String>,
 
    /// 二进制格式数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytedata: Option<Vec<u8>>,
}

impl Response {
    /// 成功响应 (back)
    pub fn success_json<T: Serialize>(data: &T) -> Self {
        Self {
            res: 0,
            errmsg: String::new(),
            kind: Some("json".to_string()),
            back: Some(serde_json::to_string(data).unwrap_or_default()),
            bytedata: None,
        }
    }

    /// 成功响应 (bytedata)
    pub fn success_bytes(data: Vec<u8>) -> Self {
        Self {
            res: 0,
            errmsg: String::new(),
            kind: Some("bytes".to_string()),
            back: None,
            bytedata: Some(data),
        }
    }

    /// 失败响应
    pub fn fail(msg: &str, code: i32) -> Self {
        Self {
            res: code,
            errmsg: msg.to_string(),
            kind: None,
            back: None,
            bytedata: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    struct TestData {
        key: String,
    }

    #[test]
    fn test_new_upinfo() {
        let up = UpInfo::new();
        assert_eq!(up.getstart, 0);
        assert_eq!(up.getnumber, 15);
        assert!(!up.mid.is_empty());
        assert!(!up.uptime.is_empty());
    }

    #[test]
    fn test_get_guest() {
        let up = UpInfo::get_guest();
        assert!(up.sid.contains("GUEST"));
        assert_eq!(up.uname, "guest");
    }

    #[test]
    fn test_default_upinfo() {
        let up = UpInfo::default_upinfo();
        assert_eq!(up.cid, "default");
        assert_eq!(up.uid, "test");
    }

    #[test]
    fn test_new_id() {
        let id1 = UpInfo::new_id();
        let id2 = UpInfo::new_id();
        assert_ne!(id1, id2);
        assert!(id1.len() == 36);
    }

    #[test]
    fn test_new_id_ts() {
        let id = UpInfo::new_id_ts();
        assert!(id.len() > 10);
    }

    #[test]
    fn test_with_api() {
        let up = UpInfo::new().with_api("api", "basic", "test");
        assert_eq!(up.apisys, "api");
        assert_eq!(up.apimicro, "basic");
        assert_eq!(up.apiobj, "test");
    }

    #[test]
    fn test_with_jsdata() {
        let up = UpInfo::new().with_jsdata(r#"{"key": "value"}"#);
        assert!(up.jsdata.is_some());
        assert_eq!(up.get_raw_data(), Some(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_get_data() {
        let up = UpInfo::new().with_jsdata(r#"{"key": "value"}"#);
        let data: TestData = up.get_data().unwrap();
        assert_eq!(data.key, "value");
    }

    #[test]
    fn test_get_data_no_data() {
        let up = UpInfo::new();
        let result: Result<TestData, UpInfoError> = up.get_data();
        assert!(matches!(result, Err(UpInfoError::NoData)));
    }

    #[test]
    fn test_in_order() {
        let up = UpInfo::new();
        assert!(up.in_order(&["col1", "col2"]));
    }

    #[test]
    fn test_response_success_json() {
        let data = TestData { key: "value".to_string() };
        let resp = Response::success_json(&data);
        assert_eq!(resp.res, 0);
        assert!(resp.back.is_some());
    }

    #[test]
    fn test_response_fail() {
        let resp = Response::fail("错误信息", -1);
        assert_eq!(resp.res, -1);
        assert_eq!(resp.errmsg, "错误信息");
    }
}