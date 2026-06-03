use serde::Deserialize;
use serde::Serialize;

/// 核心功能配置（主要是打包相关）
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct CoreConfig {
    /// 要排除的文件规则，格式为正则表达式，暂时不支持Glob表达式
    /// 匹配任意一条规则时，文件就会被忽略（忽略：管理端会当这个文件不存在一般）
    /// 编写规则时可以使用check命令快速调试是否生效
    pub exclude_rules: Vec<String>,

    /// 更新索引签名私钥，使用 PKCS#8 DER 的 base64 文本
    pub index_signature_private_key: String,

    /// 是否工作在webui模式下，还是在交互式命令行模式下
    pub webui_mode: bool,

    /// 模组平台配置
    pub mod_platform: ModPlatformConfig,
}

/// 模组平台总配置
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ModPlatformConfig {
    /// CurseForge API 配置
    pub curseforge: CurseForgeConfig,
    /// Modrinth API 配置
    pub modrinth: ModrinthConfig,
    /// 缓存配置
    pub cache: CacheConfig,
    /// 代理配置
    pub proxy: Option<ProxyConfig>,
}

impl Default for ModPlatformConfig {
    fn default() -> Self {
        Self {
            curseforge: CurseForgeConfig { api_key: String::new(), rate_limit: 5 },
            modrinth: ModrinthConfig { api_token: None, rate_limit: 30 },
            cache: CacheConfig {
                ttl_searches: 600,
                ttl_mod_detail: 3600,
                ttl_versions: 3600,
                ttl_download: 300,
                ttl_fingerprint: 86400,
                max_entries: 500,
            },
            proxy: None,
        }
    }
}

/// CurseForge API 配置
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CurseForgeConfig {
    /// API Key
    pub api_key: String,
    /// 每秒最大请求数
    pub rate_limit: u64,
}

/// Modrinth API 配置
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ModrinthConfig {
    /// API Token
    pub api_token: Option<String>,
    /// 每秒最大请求数
    pub rate_limit: u64,
}

/// 缓存配置
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CacheConfig {
    /// 搜索缓存 TTL（秒）
    pub ttl_searches: u64,
    /// 模组详情缓存 TTL（秒）
    pub ttl_mod_detail: u64,
    /// 版本列表缓存 TTL（秒）
    pub ttl_versions: u64,
    /// 下载 URL 缓存 TTL（秒）
    pub ttl_download: u64,
    /// 指纹缓存 TTL（秒）
    pub ttl_fingerprint: u64,
    /// 最大缓存条目数
    pub max_entries: usize,
}

/// 代理配置
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProxyConfig {
    /// 代理主机
    pub host: String,
    /// 代理端口
    pub port: u16,
}
