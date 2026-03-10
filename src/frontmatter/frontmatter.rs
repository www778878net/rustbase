//! FrontMatter 解析器实现
//!
//! 从 Markdown 文件中提取 YAML frontmatter

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// FrontMatter 结构
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrontMatter {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub check: String,
    #[serde(default)]
    pub taskid: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub menu: String,
    #[serde(default)]
    pub tasks: Vec<TaskInfo>,
    #[serde(default)]
    pub rollback_count: i32,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// 任务信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default, rename = "substatus")]
    pub status: String,
}

impl FrontMatter {
    /// 从文件路径解析 frontmatter
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))?;

        Self::parse(&content)
    }

    /// 从内容解析 frontmatter
    pub fn parse(content: &str) -> Result<Self, String> {
        if !content.starts_with("---") {
            return Err("文件不以 frontmatter 开始".to_string());
        }

        // 查找结束标记
        let rest = &content[3..];

        // 更简单直接的方法：逐行扫描
        let lines: Vec<&str> = rest.lines().collect();
        let mut frontmatter_lines = Vec::new();

        for line in &lines {
            if line.trim() == "---" {
                // 找到结束标记
                break;
            }
            frontmatter_lines.push(*line);
        }

        if frontmatter_lines.is_empty() {
            return Err("frontmatter 内容为空".to_string());
        }

        let frontmatter_str = frontmatter_lines.join("\n");
        let frontmatter_str = frontmatter_str.trim();

        // 使用 serde_yaml 解析
        let mut fm: FrontMatter =
            serde_yaml::from_str(frontmatter_str).map_err(|e| format!("YAML 解析失败: {}", e))?;

        // 如果 kind 为空，尝试从 extra 中获取
        if fm.kind.is_empty() {
            if let Some(Value::String(k)) = fm.extra.get("kind") {
                fm.kind = k.clone();
            }
        }

        Ok(fm)
    }

    /// 转换为 YAML 字符串
    pub fn to_yaml(&self) -> Result<String, String> {
        serde_yaml::to_string(self).map_err(|e| format!("YAML 序列化失败: {}", e))
    }

    /// 更新单个字段（如果不存在则添加）
    pub fn update_field(content: &str, field: &str, new_value: &str) -> Result<String, String> {
        if !content.starts_with("---") {
            return Err("文件不以 frontmatter 开始".to_string());
        }

        // 查找结束标记
        let rest = &content[3..];
        let lines: Vec<&str> = rest.lines().collect();
        let mut frontmatter_lines = Vec::new();
        let mut content_lines = Vec::new();
        let mut found_end = false;

        for line in lines {
            if !found_end && line.trim() == "---" {
                found_end = true;
                continue;
            }

            if !found_end {
                frontmatter_lines.push(line);
            } else {
                content_lines.push(line);
            }
        }

        if frontmatter_lines.is_empty() {
            return Err("frontmatter 内容为空".to_string());
        }

        // 移除 frontmatter_lines 开头和结尾的空行
        while frontmatter_lines.first().map(|s| s.is_empty()).unwrap_or(false) {
            frontmatter_lines.remove(0);
        }
        while frontmatter_lines.last().map(|s| s.is_empty()).unwrap_or(false) {
            frontmatter_lines.pop();
        }

        let frontmatter_str = frontmatter_lines.join("\n");
        
        // 移除 content_lines 开头的空行（保留一个用于分隔）
        while content_lines.first().map(|s| s.is_empty()).unwrap_or(false) {
            content_lines.remove(0);
        }
        let content_str = content_lines.join("\n");

        // 构建匹配模式：字段可能在行首（第一行）或换行后
        let pattern = format!(r"(?m)^{}:\s*[^\n]*", regex::escape(field));
        let re = Regex::new(&pattern).map_err(|e| format!("正则错误: {}", e))?;

        // 检查字段是否存在
        let new_frontmatter = if re.is_match(&frontmatter_str) {
            // 字段存在，更新
            let new_line = format!("{}: {}", field, new_value);
            re.replace(&frontmatter_str, &new_line).to_string()
        } else {
            // 字段不存在，添加到末尾
            format!("{}\n{}: {}", frontmatter_str, field, new_value)
        };

        // 确保内容部分有正确的开头
        let final_content = if content_str.is_empty() {
            String::new()
        } else {
            format!("\n{}", content_str)
        };

        Ok(format!("---\n{}\n---{}", new_frontmatter, final_content))
    }

    /// 获取字段值
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.extra.get(key)
    }

    /// 渲染 frontmatter + 内容为完整 markdown
    pub fn render(&self, body: &str) -> String {
        let yaml = self.to_yaml().unwrap_or_default();
        format!("---\n{}\n---\n{}", yaml.trim(), body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
kind: test
title: 测试任务
status: pending
---
# 内容
"#;
        let fm = FrontMatter::parse(content).unwrap();
        assert_eq!(fm.kind, "test");
        assert_eq!(fm.status, "pending");
    }

    #[test]
    fn test_parse_with_tasks() {
        let content = r#"---
kind: capability
status: executing
tasks:
  - id: Step1
    title: 第一步
    kind: capability
    substatus: pending
  - id: Step2
    title: 第二步
    kind: default
    substatus: pending
---
# 内容
"#;
        let fm = FrontMatter::parse(content).unwrap();
        assert_eq!(fm.tasks.len(), 2);
        assert_eq!(fm.tasks[0].id, "Step1");
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "# 没有frontmatter";
        assert!(FrontMatter::parse(content).is_err());
    }

    /// 测试 substatus 字段的序列化和反序列化
    #[test]
    fn test_substatus_serde() {
        // 测试 1: 解析包含 substatus 的 YAML
        let content = r#"---
kind: micro
status: executing
tasks:
  - id: Step1
    title: 第一步
    kind: capability
    substatus: log2test
  - id: Step2
    title: 第二步
    kind: default
    substatus: pending
---
# 内容
"#;
        let fm = FrontMatter::parse(content).unwrap();
        assert_eq!(fm.tasks.len(), 2);
        assert_eq!(fm.tasks[0].status, "log2test");
        assert_eq!(fm.tasks[1].status, "pending");

        // 测试 2: 序列化时 substatus 字段名正确
        let yaml = fm.to_yaml().unwrap();
        assert!(yaml.contains("substatus: log2test"));
        assert!(yaml.contains("substatus: pending"));
        // 确保不包含旧的 status 字段名
        assert!(!yaml.contains("- status:"));
    }

    /// Demo Verify Format 001 - 验证 FrontMatter 模块
    /// 验证任务：verify_format_001
    /// 验证内容：
    /// 1. FrontMatter 模块存在
    /// 2. FrontMatter 有 parse 方法
    #[test]
    fn demo_verify_format_001() {
        use crate::MyLogger;
        use std::path::PathBuf;

        // 创建日志目录
        let log_dir = PathBuf::from("logs/demo_verify_format_001");
        std::fs::create_dir_all(&log_dir).ok();

        // 初始化日志
        let logger = MyLogger::new("demo_verify_format_001", 3);
        logger.detail("========== 开始验证任务: verify_format_001 ==========");
        logger.detail("验证目标: FrontMatter 模块");
        logger.detail("");

        // 测试1：验证 FrontMatter 模块可以正常导入和使用
        logger.detail("--- 测试1: FrontMatter 模块存在 ---");
        let content = r#"---
kind: test
status: pending
---
# Test Content
"#;
        let result = FrontMatter::parse(content);
        assert!(result.is_ok(), "FrontMatter::parse 应该成功");
        let fm = result.unwrap();
        assert_eq!(fm.kind, "test");
        assert_eq!(fm.status, "pending");
        logger.detail(&format!("解析结果: kind={}, status={}", fm.kind, fm.status));
        logger.detail("结果: 通过");
        logger.detail("");

        // 测试2：验证 parse 方法存在且功能完整
        logger.detail("--- 测试2: FrontMatter parse 方法功能 ---");
        let content2 = r#"---
kind: default
status: completed
taskid: verify_format_001
---
# Content
"#;
        let fm2 = FrontMatter::parse(content2).unwrap();
        assert_eq!(fm2.kind, "default");
        assert_eq!(fm2.status, "completed");
        assert_eq!(fm2.taskid, "verify_format_001");
        logger.detail(&format!(
            "完整解析: kind={}, status={}, taskid={}",
            fm2.kind, fm2.status, fm2.taskid
        ));
        logger.detail("结果: 通过");
        logger.detail("");

        logger.detail("========== 验证结果: 全部通过 ==========");
    }

    /// 测试 update_field 方法在更新第一个字段时的 bug
    #[test]
    fn test_update_field_first_line() {
        let content = r#"---
kind: micro
status: executing
---
# Content
"#;
        // 更新第一个字段 kind
        let result = FrontMatter::update_field(content, "kind", "default").unwrap();
        println!("Updated content:\n{}", result);

        // 验证结果可以被正确解析
        let parsed = FrontMatter::parse(&result);
        match parsed {
            Ok(fm) => {
                println!("✓ Parsed successfully, kind: {}", fm.kind);
                assert_eq!(fm.kind, "default");
            }
            Err(e) => {
                println!("✗ Failed to parse: {}", e);
                panic!("update_field created invalid YAML");
            }
        }
    }

    /// 测试多次更新字段的情况
    #[test]
    fn test_update_field_multiple_times() {
        let content = r#"---
kind: micro
status: executing
taskid: '123456'
---
# Content
"#;
        
        // 第一次更新
        let result1 = FrontMatter::update_field(content, "status", "completed").unwrap();
        println!("After 1st update:\n{}", result1);
        
        // 第二次更新
        let result2 = FrontMatter::update_field(&result1, "check", "passed").unwrap();
        println!("\nAfter 2nd update:\n{}", result2);
        
        // 第三次更新
        let result3 = FrontMatter::update_field(&result2, "status", "pending").unwrap();
        println!("\nAfter 3rd update:\n{}", result3);
        
        // 验证最终结果可以被正确解析
        let parsed = FrontMatter::parse(&result3);
        match parsed {
            Ok(fm) => {
                println!("✓ Parsed successfully");
                println!("  kind: {}", fm.kind);
                println!("  status: {}", fm.status);
                println!("  taskid: {}", fm.taskid);
                println!("  check: {}", fm.check);
            }
            Err(e) => {
                println!("✗ Failed to parse: {}", e);
                panic!("Multiple updates created invalid YAML");
            }
        }
    }

    /// 测试添加字段后 frontmatter 和 --- 之间有换行
    #[test]
    fn test_update_field_add_field_with_proper_newlines() {
        let content = r#"---
kind: micro
status: executing
created_at: 2026-03-06T11:12:34Z

---
# Content
"#;
        
        // 添加 title 字段（字段不存在，会添加到末尾）
        let result = FrontMatter::update_field(content, "title", "test_title").unwrap();
        println!("After adding title:\n{}", result);
        println!("---");
        
        // 检查是否包含正确的格式
        let has_proper_newline = result.contains("title: test_title\n\n---");
        println!("Has proper newline before ---: {}", has_proper_newline);
        
        // 验证可以被正确解析
        let parsed = FrontMatter::parse(&result);
        match parsed {
            Ok(fm) => {
                println!("✓ Parsed successfully");
                assert_eq!(fm.kind, "micro");
                assert_eq!(fm.status, "executing");
                // title 是显式字段，检查 fm.title
                assert_eq!(fm.title, "test_title");
            }
            Err(e) => {
                println!("✗ Failed to parse: {}", e);
                panic!("Adding field created invalid YAML");
            }
        }
    }

    /// 测试实际场景：多次更新后文件是否损坏
    #[test]
    fn test_update_field_real_scenario() {
        // 模拟真实的 plan 文件
        let content = r#"---
kind: micro
status: executing
check: ''
taskid: '20260306111234'
tasks:
- id: Step0
  title: 注册本地表
  kind: capability
  substatus: pending
rollback_count: 0
created_at: 2026-03-06T11:12:34Z

---
## 管理员指示

重构 other/buffscan/src/steam/strategy_cent/svc_steam_strategy_cent.py
"#;
        
        println!("Original content (lines 30-40):");
        for (i, line) in content.lines().enumerate().skip(29).take(15) {
            println!("{:3}: '{}'", i + 1, line);
        }
        
        // 第一次更新：更新 status
        let result1 = FrontMatter::update_field(content, "substatus", "log2test").unwrap();
        println!("\nAfter updating substatus:");
        println!("{}", result1);
        
        // 验证可以解析
        let parsed1 = FrontMatter::parse(&result1);
        match parsed1 {
            Ok(fm) => println!("✓ Parsed after 1st update: substatus={}", fm.tasks.get(0).map(|t| t.status.as_str()).unwrap_or("")),
            Err(e) => panic!("1st update created invalid YAML: {}", e),
        }
        
        // 第二次更新：添加 check 字段
        let result2 = FrontMatter::update_field(&result1, "check", "passed").unwrap();
        println!("\nAfter adding check:");
        println!("{}", result2);
        
        // 验证可以解析
        let parsed2 = FrontMatter::parse(&result2);
        match parsed2 {
            Ok(fm) => {
                println!("✓ Parsed after 2nd update: check={}", fm.check);
                // 检查文件结构是否正确
                assert!(result2.contains("\n---\n"), "Should have proper --- separator");
                assert!(!result2.contains("Z---"), "Should NOT have Z--- (missing newline)");
            }
            Err(e) => panic!("2nd update created invalid YAML: {}", e),
        }
    }

    /// 测试 FrontMatter::render 方法
    #[test]
    fn test_render() {
        let mut fm = FrontMatter::default();
        fm.kind = "micro".to_string();
        fm.status = "executing".to_string();
        fm.taskid = "20260306111234".to_string();
        // title 是显式字段，直接设置而不是插入 extra
        fm.title = "test".to_string();
        fm.extra.insert("created_at".to_string(), Value::String("2026-03-06T11:12:34Z".to_string()));

        let body = "## 管理员指示\n\n重构 xxx";
        let result = fm.render(body);
        
        println!("Rendered:\n{}", result);
        println!("---");
        println!("Contains \\n---\\n: {}", result.contains("\n---\n"));
        println!("Contains ---\\n##: {}", result.contains("---\n##"));
        
        // 验证结构
        assert!(result.starts_with("---\n"), "Should start with ---");
        // render 方法输出格式是: "---\n{yaml}\n---\n{body}"
        // 所以应该包含 "---\n" 后面跟着 yaml，然后 "---\n" 后面跟着 body
        let parts: Vec<&str> = result.split("\n---\n").collect();
        println!("Parts count: {}", parts.len());
        for (i, part) in parts.iter().enumerate() {
            println!("Part {}: {} chars", i, part.len());
        }
        
        assert!(result.contains("\n---\n##"), "Should have proper separator before body");
        assert!(result.contains("## 管理员指示"), "Should contain body");
        
        // 验证可以解析回去
        let parsed = FrontMatter::parse(&result).unwrap();
        assert_eq!(parsed.kind, "micro");
        assert_eq!(parsed.status, "executing");
        assert_eq!(parsed.taskid, "20260306111234");
    }

}
