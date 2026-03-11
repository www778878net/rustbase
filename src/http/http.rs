//! HTTP请求工具实现
//!
//! 提供统一的 HTTP 请求处理能力

use crate::get_logger;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use std::time::Duration;
use ureq::{Agent, AgentBuilder, Proxy};

/// HTTP响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// 结果码：0成功，-1失败
    pub res: i32,
    /// 错误信息
    pub errmsg: String,
    /// 响应数据
    pub data: Option<ResponseData>,
}

/// 响应数据详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    /// HTTP状态码
    pub status_code: u16,
    /// 响应内容
    pub response: Value,
    /// 内容类型：json 或 text
    pub kind: String,
    /// Cookies（仅GET请求）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<HashMap<String, String>>,
}

/// HTTP请求工具类
pub struct HttpHelper;

impl HttpHelper {
    /// 从环境变量获取代理地址
    /// 
    /// 读取顺序：HTTP_PROXY -> http_proxy
    /// 默认返回 None（不使用代理）
    fn get_proxy_from_env() -> Option<String> {
        std::env::var("HTTP_PROXY")
            .or_else(|_| std::env::var("http_proxy"))
            .ok()
    }

    /// 创建HTTP Agent
    fn create_agent(timeout_secs: u64, proxy: Option<&str>) -> Result<Agent, String> {
        let mut builder = AgentBuilder::new()
            .timeout_read(Duration::from_secs(timeout_secs))
            .timeout_write(Duration::from_secs(timeout_secs));

        if let Some(proxy_url) = proxy {
            let p = Proxy::new(proxy_url)
                .map_err(|e| format!("代理配置失败: {}", e))?;
            builder = builder.proxy(p);
        }

        Ok(builder.build())
    }

    /// 发送HTTP GET请求
    pub fn get(
        url: &str,
        headers: Option<&[(&str, &str)]>,
        params: Option<&[(&str, &str)]>,
        use_proxy: bool,
        proxy: Option<&str>,
        timeout: u64,
        max_retries: u32,
    ) -> HttpResponse {
        let final_proxy = if use_proxy {
            proxy.map(|s| s.to_string()).or_else(|| Self::get_proxy_from_env())
        } else {
            None
        };

        for attempt in 0..=max_retries {
            match Self::do_get(url, headers, params, final_proxy.as_deref(), timeout) {
                Ok(response) => return response,
                Err(errmsg) => {
                    if attempt < max_retries {
                        std::thread::sleep(Duration::from_secs(2));
                        continue;
                    }
                    return HttpResponse {
                        res: -1,
                        errmsg,
                        data: None,
                    };
                }
            }
        }

        HttpResponse {
            res: -1,
            errmsg: "所有重试均失败".to_string(),
            data: None,
        }
    }

    /// 执行GET请求
    fn do_get(
        url: &str,
        headers: Option<&[(&str, &str)]>,
        params: Option<&[(&str, &str)]>,
        proxy: Option<&str>,
        timeout: u64,
    ) -> Result<HttpResponse, String> {
        let logger = get_logger("HTTP", 3);
        logger.detail(&format!("[GET] {}", url));
        
        let agent = Self::create_agent(timeout, proxy)?;
        let mut request = agent.get(url);

        if let Some(p) = params {
            for (key, value) in p {
                request = request.query(key, value);
            }
        }

        if let Some(h) = headers {
            for (key, value) in h {
                request = request.set(key, value);
            }
        }

        let response = request.call()
            .map_err(|e| format!("请求失败: {}", e))?;

        let status_code = response.status();
        let cookies = Self::extract_cookies(&response);
        let (response_content, kind) = Self::read_response(response)?;
        
        let response_preview = if response_content.to_string().chars().count() > 500 {
            format!("{}...(截断)", response_content.to_string().chars().take(500).collect::<String>())
        } else {
            response_content.to_string()
        };
        logger.detail(&format!("[GET] 响应 status={} body={}", status_code, response_preview));

        if status_code < 200 || status_code >= 300 {
            return Ok(HttpResponse {
                res: -1,
                errmsg: response_content.to_string(),
                data: Some(ResponseData {
                    status_code,
                    response: response_content,
                    kind,
                    cookies: None,
                }),
            });
        }

        Ok(HttpResponse {
            res: 0,
            errmsg: String::new(),
            data: Some(ResponseData {
                status_code,
                response: response_content,
                kind,
                cookies: Some(cookies),
            }),
        })
    }

    /// 发送HTTP POST请求
    pub fn post(
        url: &str,
        data: Option<&[(&str, &str)]>,
        json_data: Option<&Value>,
        headers: Option<&[(&str, &str)]>,
        use_proxy: bool,
        proxy: Option<&str>,
        timeout: u64,
        max_retries: u32,
    ) -> HttpResponse {
        let final_proxy = if use_proxy {
            proxy.map(|s| s.to_string()).or_else(|| Self::get_proxy_from_env())
        } else {
            None
        };

        for attempt in 0..=max_retries {
            match Self::do_post(url, data, json_data, headers, final_proxy.as_deref(), timeout) {
                Ok(response) => return response,
                Err(errmsg) => {
                    if attempt < max_retries {
                        std::thread::sleep(Duration::from_secs(2));
                        continue;
                    }
                    return HttpResponse {
                        res: -1,
                        errmsg,
                        data: None,
                    };
                }
            }
        }

        HttpResponse {
            res: -1,
            errmsg: "所有重试均失败".to_string(),
            data: None,
        }
    }

    /// 执行POST请求
    fn do_post(
        url: &str,
        data: Option<&[(&str, &str)]>,
        json_data: Option<&Value>,
        headers: Option<&[(&str, &str)]>,
        proxy: Option<&str>,
        timeout: u64,
    ) -> Result<HttpResponse, String> {
        let logger = get_logger("HTTP", 3);
        logger.detail(&format!("[POST] URL: {}", url));
        if let Some(json) = json_data {
            let json_str = json.to_string();
            if json_str.chars().count() > 500 {
                logger.detail(&format!("[POST] JSON: {}...(截断)", json_str.chars().take(500).collect::<String>()));
            } else {
                logger.detail(&format!("[POST] JSON: {}", json_str));
            }
        }
        if let Some(form_data) = data {
            logger.detail(&format!("[POST] FORM: {:?}", form_data));
        }
        
        let agent = Self::create_agent(timeout, proxy)?;
        let mut request = agent.post(url);

        if let Some(h) = headers {
            for (key, value) in h {
                request = request.set(key, value);
            }
        }

        let response = if let Some(json) = json_data {
            request
                .set("Content-Type", "application/json")
                .send_json(json)
        } else if let Some(form_data) = data {
            request
                .set("Content-Type", "application/x-www-form-urlencoded")
                .send_form(form_data)
        } else {
            request.call()
        };

        let response = match response {
            Ok(r) => r,
            Err(ureq::Error::Status(code, resp)) => {
                let (body, _) = Self::read_response(resp).unwrap_or((Value::String(String::new()), "text".to_string()));
                logger.error(&format!("[POST] HTTP错误: {} status={} body={}", url, code, body));
                return Ok(HttpResponse {
                    res: -1,
                    errmsg: body.to_string(),
                    data: Some(ResponseData {
                        status_code: code as u16,
                        response: body,
                        kind: "json".to_string(),
                        cookies: None,
                    }),
                });
            }
            Err(e) => {
                let err_msg = e.to_string();
                logger.error(&format!("[POST] 网络错误: {} error={}", url, err_msg));
                return Ok(HttpResponse {
                    res: -1,
                    errmsg: err_msg.clone(),
                    data: Some(ResponseData {
                        status_code: 0,
                        response: Value::String(err_msg),
                        kind: "text".to_string(),
                        cookies: None,
                    }),
                });
            }
        };

        let status_code = response.status();
        let (response_content, kind) = Self::read_response(response).map_err(|e| {
            logger.error(&format!("[POST] 读取响应失败: {}: {}", url, e));
            format!("读取响应失败: {}: {}", url, e)
        })?;
        
        let response_preview = if response_content.to_string().chars().count() > 500 {
            format!("{}...(截断)", response_content.to_string().chars().take(500).collect::<String>())
        } else {
            response_content.to_string()
        };
        logger.detail(&format!("[POST] 响应 status={} body={}", status_code, response_preview));

        if status_code < 200 || status_code >= 300 {
            let server_error = response_content.to_string();
            let errmsg = if server_error.is_empty() {
                format!("HTTP请求失败，状态码: {}", status_code)
            } else {
                format!("HTTP {} 错误: {}", status_code, server_error)
            };
            return Ok(HttpResponse {
                res: -1,
                errmsg,
                data: Some(ResponseData {
                    status_code,
                    response: response_content,
                    kind,
                    cookies: None,
                }),
            });
        }

        Ok(HttpResponse {
            res: 0,
            errmsg: String::new(),
            data: Some(ResponseData {
                status_code,
                response: response_content,
                kind,
                cookies: None,
            }),
        })
    }

    /// 读取响应内容
    fn read_response(response: ureq::Response) -> Result<(Value, String), String> {
        let mut text = String::new();
        response.into_reader()
            .read_to_string(&mut text)
            .map_err(|e| format!("读取响应失败: {}", e))?;

        if let Ok(json) = serde_json::from_str::<Value>(&text) {
            Ok((json, "json".to_string()))
        } else {
            Ok((Value::String(text), "text".to_string()))
        }
    }

    /// 从响应中提取Cookies
    fn extract_cookies(response: &ureq::Response) -> HashMap<String, String> {
        let mut cookies = HashMap::new();

        for value in response.all("Set-Cookie") {
            if let Some(eq_pos) = value.find('=') {
                let name = value[..eq_pos].trim();
                let rest = &value[eq_pos + 1..];
                let value = if let Some(semi_pos) = rest.find(';') {
                    rest[..semi_pos].trim()
                } else {
                    rest.trim()
                };
                cookies.insert(name.to_string(), value.to_string());
            }
        }

        cookies
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_simple() {
        let result = HttpHelper::get(
            "https://httpbin.org/get",
            None,
            None,
            false,
            None,
            30,
            0
        );

        println!("Result: {:?}", result);
        assert_eq!(result.res, 0);
    }

    #[test]
    fn test_get_with_params() {
        let result = HttpHelper::get(
            "https://httpbin.org/get",
            Some(&[("User-Agent", "RustTest/1.0")]),
            Some(&[("foo", "bar"), ("baz", "qux")]),
            false,
            None,
            30,
            0
        );

        println!("Result: {:?}", result);
        assert_eq!(result.res, 0);
    }

    #[test]
    fn test_post_json() {
        let json_data = json!({
            "key": "value",
            "number": 42
        });

        let result = HttpHelper::post(
            "https://httpbin.org/post",
            None,
            Some(&json_data),
            None,
            false,
            None,
            30,
            0
        );

        println!("Result: {:?}", result);
        assert_eq!(result.res, 0);
    }

    #[test]
    fn test_post_form() {
        let form = [("username", "test"), ("password", "secret")];

        let result = HttpHelper::post(
            "https://httpbin.org/post",
            Some(&form),
            None,
            None,
            false,
            None,
            30,
            0
        );

        println!("Result: {:?}", result);
        assert_eq!(result.res, 0);
    }

    #[test]
    fn test_timeout() {
        let result = HttpHelper::get(
            "https://httpbin.org/delay/10",
            None,
            None,
            false,
            None,
            2,
            0
        );

        println!("Result: {:?}", result);
        assert_eq!(result.res, -1);
    }

    #[test]
    fn test_retry() {
        let result = HttpHelper::get(
            "https://httpbin.org/status/500",
            None,
            None,
            false,
            None,
            5,
            2
        );

        println!("Result: {:?}", result);
        assert_eq!(result.res, -1);
    }

    // 集成测试：依赖外部 API，默认跳过
    #[test]
    #[ignore]
    fn test_steam_strategy_holding_api() {
        let url = "http://log.778878.net/apisteam/strategy/steam_strategy_holding/get?sid=15d647fd-946d-93f6-2413-1a702f365816&getnumber=5&pars=steam_rent&pars=mypc";
        let result = HttpHelper::get(
            url,
            None,
            None,
            false,
            None,
            30,
            0
        );

        println!("Result: {:?}", result);
        println!("Response: {:?}", result.data.as_ref().map(|d| &d.response));
        assert_eq!(result.res, 0);
    }

    // 集成测试：依赖外部 API，默认跳过
    #[test]
    #[ignore]
    fn test_steam_strategy_holding_no_worker() {
        let url = "http://log.778878.net/apisteam/strategy/steam_strategy_holding/get?sid=15d647fd-946d-93f6-2413-1a702f365816&getnumber=5&pars=steam_rent&pars=";
        let result = HttpHelper::get(
            url,
            None,
            None,
            false,
            None,
            30,
            0
        );

        println!("Result (empty worker): {:?}", result);
        if let Some(data) = &result.data {
            let response = &data.response;
            println!("Response: {}", serde_json::to_string_pretty(response).unwrap());
        }
        assert_eq!(result.res, 0);
    }
}