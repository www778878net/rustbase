//! ProjectPath - 项目路径查找工具实现
//!
//! 提供项目根目录查找、默认路径生成、配置加载等能力

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

/// 环境类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Test,
    Production,
}

impl Default for Environment {
    fn default() -> Self {
        Self::from_env()
    }
}

impl Environment {
    /// 从 APP_ENV 环境变量获取环境
    pub fn from_env() -> Self {
        match std::env::var("APP_ENV")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "development" | "dev" => Environment::Development,
            "test" | "testing" => Environment::Test,
            "production" | "prod" => Environment::Production,
            _ => Environment::Development,
        }
    }

    /// 获取环境名称
    pub fn name(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Test => "test",
            Environment::Production => "production",
        }
    }
}

/// 项目路径查找结果
#[derive(Debug, Clone)]
pub struct ProjectPath {
    /// 项目根目录（包含 docs 或 .claude 的目录）
    root: PathBuf,
    /// 当前环境
    environment: Environment,
}

impl ProjectPath {
    /// 查找项目根目录
    ///
    /// 从当前工作目录开始，向上查找 docs 或 .claude 目录
    /// 找到的第一个即为项目根目录
    pub fn find() -> Result<Self, String> {
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("获取当前目录失败: {}", e))?;

        Self::find_from(&current_dir)
    }

    /// 从指定路径开始查找项目根目录
    pub fn find_from(start: &PathBuf) -> Result<Self, String> {
        let mut path = start.clone();

        loop {
            let docs_path = path.join("docs");
            let claude_path = path.join(".claude");

            if docs_path.exists() || claude_path.exists() {
                return Ok(Self {
                    root: path,
                    environment: Environment::from_env(),
                });
            }

            // 向上一级
            match path.parent() {
                Some(parent) => path = parent.to_path_buf(),
                None => break,
            }
        }

        Err(format!(
            "未找到项目根目录（未找到 docs 或 .claude 目录）, 起始路径: {:?}",
            start
        ))
    }

    /// 获取项目根目录
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// 获取当前环境
    pub fn environment(&self) -> Environment {
        self.environment
    }

    /// 获取 docs 目录
    pub fn docs(&self) -> PathBuf {
        self.root.join("docs")
    }

    /// 获取 docs/config 目录
    pub fn docs_config(&self) -> PathBuf {
        self.root.join("docs").join("config")
    }

    /// 获取配置文件路径 (docs/config/{filename})
    pub fn config_file(&self, filename: &str) -> PathBuf {
        self.docs_config().join(filename)
    }

    /// 获取环境配置文件路径 (docs/config/{env}.ini)
    pub fn env_config_file(&self) -> PathBuf {
        self.docs_config().join(format!("{}.ini", self.environment.name()))
    }

    /// 获取本地数据库路径 (docs/config/local.db)
    pub fn local_db(&self) -> PathBuf {
        self.docs_config().join("local.db")
    }

    /// 读取简单文本配置文件
    pub fn read_text_config(&self, filename: &str, env_key: &str) -> Option<String> {
        // 环境变量优先
        if let Ok(val) = std::env::var(env_key) {
            return Some(val);
        }

        // 读取文件内容
        let config_path = self.config_file(filename);
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                let trimmed = content.trim().to_string();
                if !trimmed.is_empty() {
                    return Some(trimmed);
                }
            }
        }
        None
    }

    /// 获取 Worker 名称
    pub fn worker_name(&self) -> Option<String> {
        self.read_text_config("worker.txt", "WORKER_NAME")
    }

    /// 加载 INI 配置文件
    pub fn load_ini_config(&self) -> Result<HashMap<String, HashMap<String, String>>, String> {
        let config_path = self.env_config_file();

        if !config_path.exists() {
            return Err(format!("配置文件不存在: {}", config_path.to_string_lossy()));
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;

        let mut config: HashMap<String, HashMap<String, String>> = HashMap::new();
        let mut current_section = "default".to_string();

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len()-1].trim().to_string();
                config.entry(current_section.clone()).or_insert_with(HashMap::new);
                continue;
            }

            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();
                config
                    .entry(current_section.clone())
                    .or_insert_with(HashMap::new)
                    .insert(key, value);
            }
        }

        Ok(config)
    }

    /// 从 INI 配置文件读取值
    pub fn read_ini_value(&self, section: &str, key: &str) -> Option<String> {
        let env_key = key.to_uppercase();
        if let Ok(val) = std::env::var(&env_key) {
            return Some(val);
        }

        self.load_ini_config()
            .ok()?
            .get(section)?
            .get(key)
            .cloned()
    }

    /// 加载配置文件（key=value 格式）
    pub fn load_config(&self, filename: &str) -> Result<HashMap<String, String>, String> {
        let config_path = self.config_file(filename);

        if !config_path.exists() {
            return Err(format!("配置文件不存在: {}", config_path.to_string_lossy()));
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;

        let mut config = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();
                config.insert(key, value);
            }
        }

        Ok(config)
    }

    /// 获取配置值
    pub fn get_config(&self, filename: &str, key: &str) -> Option<String> {
        let env_key = key.to_uppercase();
        if let Ok(val) = std::env::var(&env_key) {
            return Some(val);
        }

        self.load_config(filename).ok()?.get(key).cloned()
    }

    /// 获取 .claude 目录
    pub fn claude(&self) -> PathBuf {
        self.root.join(".claude")
    }

    /// 获取 logs 目录
    pub fn logs(&self) -> PathBuf {
        self.root.join("logs")
    }

    /// 获取 tmp 目录
    pub fn tmp(&self) -> PathBuf {
        self.root.join("tmp")
    }

    /// 获取 memory78 目录
    pub fn memory78(&self) -> PathBuf {
        self.root.join("memory78")
    }

    /// 获取 data 目录（子GIT，存放 input/check 等测试数据）
    pub fn data(&self) -> PathBuf {
        self.root.join("data")
    }

    /// 获取 input 文件路径
    /// 例如: module="base", class="project_path" -> data/base/src/project_path/project_path.input
    pub fn data_input(&self, module: &str, class: &str) -> PathBuf {
        self.data()
            .join(module)
            .join("src")
            .join(class)
            .join(format!("{}.input", class))
    }

    /// 获取 check 文件路径
    pub fn data_check(&self, module: &str, class: &str) -> PathBuf {
        self.data()
            .join(module)
            .join("src")
            .join(class)
            .join(format!("{}.check", class))
    }

    /// 拼接任意路径
    pub fn join(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        self.root.to_string_lossy().to_string()
    }
}

impl Default for ProjectPath {
    fn default() -> Self {
        Self::find().unwrap_or_else(|_| Self {
            root: std::env::current_dir().unwrap_or_default(),
            environment: Environment::from_env(),
        })
    }
}