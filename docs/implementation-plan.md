# MCPATCH2 v2.1 多平台模组下载支持 — 分阶段实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 subagent-driven-development 或 executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 为 MCPATCH2 增加 CurseForge 和 Modrinth 模组平台支持，在 `pack` 阶段自动识别 JAR 文件并替换为 CDN 下载 URL，减少服务器带宽消耗。

**架构：**

- **Rust 服务端**：新增 `modplatform` 模块（Provider Trait + CF/MR 实现 + 缓存 + 限流器），扩展 `task/pack` 流程实现 ModRouter 模组识别，新增 REST API 端点
- **Java 客户端**：在 `FileChange.UpdateFile` 中新增 `source` + `downloadUrl` 字段，`Work.java` 中增加 CDN 下载分支，复用 `Servers.multipleAvailableServers()` 实现双源降级
- **WebUI 前端**：新增模组平台搜索、预览页面

**技术栈：**

- Rust: axum, reqwest, serde, sha2, tokio, murmur2
- Java: OkHttp, JSON 解析
- Frontend: React, React Router, Axios, Ant Design

### 阶段 1：核心基础设施 — modplatform 模块骨架

**文件：**

- 创建：`McPatch2/manager/src/modplatform/mod.rs`
- 创建：`McPatch2/manager/src/modplatform/types.rs`
- 创建：`McPatch2/manager/src/modplatform/error.rs`
- 创建：`McPatch2/manager/src/modplatform/rate_limiter.rs`
- 修改：`McPatch2/manager/src/main.rs`
- [ ] **步骤 1：创建** **`modplatform/types.rs`** — 定义统一数据模型

```rust
use serde::{Deserialize, Serialize};
use std::fmt;

/// 平台标识符
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlatformId {
    CurseForge,
    Modrinth,
}

impl fmt::Display for PlatformId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformId::CurseForge => write!(f, "curseforge"),
            PlatformId::Modrinth => write!(f, "modrinth"),
        }
    }
}

/// 文件来源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSource {
    pub platform: PlatformId,
    pub mod_id: String,
    pub file_id: Option<String>,
    pub download_url: Option<String>,
    pub resolved: bool,
}

/// 模组搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSearchResult {
    pub platform: PlatformId,
    pub mod_id: String,
    pub name: String,
    pub slug: String,
    pub summary: String,
    pub logo_url: Option<String>,
    pub game_versions: Vec<String>,
    pub mod_loaders: Vec<String>,
}

/// 模组版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVersionInfo {
    pub platform: PlatformId,
    pub mod_id: String,
    pub file_id: String,
    pub name: String,
    pub game_version: String,
    pub mod_loader: Option<String>,
    pub download_url: String,
    pub filename: String,
    pub sha1_hash: Option<String>,
}

/// 模组详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDetail {
    pub platform: PlatformId,
    pub mod_id: String,
    pub name: String,
    pub slug: String,
    pub summary: String,
    pub logo_url: Option<String>,
    pub game_versions: Vec<String>,
    pub mod_loaders: Vec<String>,
    pub download_count: i64,
}

/// CDN 下载条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnDownloadEntry {
    pub file_path: String,
    pub source: String,
    pub download_url: String,
    pub filename: String,
}
```

- [ ] **步骤 2：创建** **`modplatform/error.rs`** — 平台相关错误类型

```rust
use std::fmt;

#[derive(Debug)]
pub enum ModPlatformError {
    NetworkError(String),
    AuthFailed(String),
    RateLimited { retry_after_ms: u64 },
    NotFound(String),
    ParseError(String),
    OfflineMode,
    CacheDisabled,
    Internal(String),
}

impl fmt::Display for ModPlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModPlatformError::NetworkError(msg) => write!(f, "网络错误: {}", msg),
            ModPlatformError::AuthFailed(msg) => write!(f, "认证失败: {}", msg),
            ModPlatformError::RateLimited { retry_after_ms } => write!(f, "请求过多，请在 {}ms 后重试", retry_after_ms),
            ModPlatformError::NotFound(msg) => write!(f, "未找到: {}", msg),
            ModPlatformError::ParseError(msg) => write!(f, "数据解析失败: {}", msg),
            ModPlatformError::OfflineMode => write!(f, "离线模式，跳过 API 调用"),
            ModPlatformError::CacheDisabled => write!(f, "缓存功能未启用"),
            ModPlatformError::Internal(msg) => write!(f, "内部错误: {}", msg),
        }
    }
}

impl std::error::Error for ModPlatformError {}
```

- [ ] **步骤 3：创建** **`modplatform/rate_limiter.rs`** — 令牌桶限流器

```rust
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// 令牌桶限流器
pub struct RateLimiter {
    tokens: Arc<Mutex<TokenBucket>>,
}

struct TokenBucket {
    capacity: u64,
    tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new(capacity: u64, refill_per_second: f64) -> Self {
        Self {
            tokens: Arc::new(Mutex::new(TokenBucket {
                capacity,
                tokens: capacity as f64,
                refill_rate: refill_per_second / 1000.0,
                last_refill: Instant::now(),
            })),
        }
    }

    /// 尝试获取一个令牌，返回等待时间（毫秒）
    pub async fn acquire(&self) -> Option<Duration> {
        let mut bucket = self.tokens.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(bucket.last_refill).as_millis() as f64;
        bucket.tokens = (bucket.tokens + elapsed * bucket.refill_rate).min(bucket.capacity as f64);
        bucket.last_refill = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            None
        } else {
            let wait_ms = ((1.0 - bucket.tokens) / bucket.refill_rate) as u64;
            Some(Duration::from_millis(wait_ms.min(5000)))
        }
    }
}
```

- [ ] **步骤 4：创建** **`modplatform/mod.rs`** — 模块入口与 Provider Trait

```rust
pub mod types;
pub mod error;
pub mod rate_limiter;

use std::sync::Arc;
use async_trait::async_trait;
use crate::modplatform::error::ModPlatformError;
use crate::modplatform::types::*;

/// 模组平台 Provider 抽象 Trait
#[async_trait]
pub trait ModPlatformProvider: Send + Sync {
    fn platform_id(&self) -> PlatformId;

    /// 搜索模组
    async fn search(&self, query: &str, game_version: Option<&str>, mod_loader: Option<&str>) -> Result<Vec<ModSearchResult>, ModPlatformError>;

    /// 获取模组详情
    async fn get_mod(&self, mod_id: &str) -> Result<ModDetail, ModPlatformError>;

    /// 获取模组版本列表
    async fn get_versions(&self, mod_id: &str, game_version: Option<&str>, mod_loader: Option<&str>) -> Result<Vec<ModVersionInfo>, ModPlatformError>;

    /// 获取下载 URL
    async fn get_download_url(&self, mod_id: &str, file_id: &str) -> Result<String, ModPlatformError>;

    /// 批量指纹匹配（仅 CurseForge）
    async fn batch_resolve_fingerprints(&self, fingerprints: &[i64]) -> Result<Vec<(i64, FileSource)>, ModPlatformError> {
        let _ = fingerprints;
        Err(ModPlatformError::Internal("not supported by this platform".into()))
    }

    /// SHA-1 哈希查找（仅 Modrinth）
    async fn resolve_hash(&self, sha1: &str) -> Result<Option<FileSource>, ModPlatformError> {
        let _ = sha1;
        Err(ModPlatformError::Internal("not supported by this platform".into()))
    }
}

pub type ArcProvider = Arc<dyn ModPlatformProvider>;
```

- [ ] **步骤 5：修改** **`main.rs`** — 注册 modplatform 模块

添加到 main.rs 的 mod 声明区：

```rust
pub mod modplatform;
```

- [ ] **步骤 6：验证编译通过**

```bash
cd McPatch2/manager
cargo check 2>&1
```

- [ ] **步骤 7：Commit**

```bash
git add McPatch2/manager/src/modplatform/ McPatch2/manager/src/main.rs
git commit -m "feat(modplatform): add core module skeleton with types, errors, rate limiter, and Provider trait"
```

***

### 阶段 2：指纹算法 — CurseForge MurmurHash2 + SHA-1 工具

**文件：**

- 创建：`McPatch2/manager/src/core/fingerprint.rs`
- 修改：`McPatch2/manager/src/core/mod.rs`
- [ ] **步骤 1：创建** **`core/fingerprint.rs`** — CurseForge 指纹算法

```rust
/// CurseForge 使用的指纹算法
/// 基于 MurmurHash2 64 位变体，搭配特定种子值

const CF_SEED: u64 = 1;
const CF_MULTIPLIER: u64 = 0x5bd1e995;
const CF_RIGHT_SHIFT: u8 = 47;

/// 计算单个文件的 CurseForge 指纹（64 位）
pub fn calculate_fingerprint(data: &[u8]) -> i64 {
    let len = data.len();
    let mut hash = CF_SEED ^ (len as u64).wrapping_mul(CF_MULTIPLIER);
    let mut i = 0;

    // 每 8 字节处理一块
    while i + 8 <= len {
        let mut k = u64::from_le_bytes([
            data[i],
            data[i + 1],
            data[i + 2],
            data[i + 3],
            data[i + 4],
            data[i + 5],
            data[i + 6],
            data[i + 7],
        ]);

        k = k.wrapping_mul(CF_MULTIPLIER);
        k ^= k >> CF_RIGHT_SHIFT;
        k = k.wrapping_mul(CF_MULTIPLIER);
        hash ^= k;
        hash = hash.wrapping_mul(CF_MULTIPLIER);
        i += 8;
    }

    // 处理剩余字节
    let remaining = len - i;
    if remaining > 0 {
        let mut k = 0u64;
        for j in (0..remaining).rev() {
            k ^= (data[i + j] as u64) << (j * 8);
        }
        hash ^= k;
        hash = hash.wrapping_mul(CF_MULTIPLIER);
    }

    hash ^= hash >> CF_RIGHT_SHIFT;
    hash = hash.wrapping_mul(CF_MULTIPLIER);
    hash ^= hash >> CF_RIGHT_SHIFT;

    hash as i64
}

/// 计算文件的 SHA-1 哈希（用于 Modrinth 查找）
pub fn calculate_sha1(data: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_file() {
        let fp = calculate_fingerprint(b"");
        assert_ne!(fp, 0);
    }

    #[test]
    fn test_known_data() {
        let data = b"hello curseforge fingerprint test";
        let fp = calculate_fingerprint(data);
        // 验证确定性：相同输入产出相同输出
        let fp2 = calculate_fingerprint(data);
        assert_eq!(fp, fp2);
    }

    #[test]
    fn test_sha1() {
        let hash = calculate_sha1(b"hello world");
        assert_eq!(hash.len(), 40);
        assert_eq!(hash, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
    }
}
```

- [ ] **步骤 2：修改** **`core/mod.rs`** — 注册指纹模块

```rust
pub mod fingerprint;
```

- [ ] **步骤 3：验证编译 + 测试通过**

```bash
cd McPatch2/manager
cargo test -- core::fingerprint 2>&1
```

- [ ] **步骤 4：Commit**

```bash
git add McPatch2/manager/src/core/fingerprint.rs McPatch2/manager/src/core/mod.rs
git commit -m "feat(core): add CurseForge fingerprint algorithm and SHA-1 utility"
```

***

### 阶段 3：缓存层

**文件：**

- 创建：`McPatch2/manager/src/modplatform/cache.rs`
- 修改：`McPatch2/manager/src/modplatform/mod.rs`
- [ ] **步骤 1：创建** **`modplatform/cache.rs`**

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::modplatform::types::*;

/// 缓存条目
#[derive(Clone)]
struct CacheEntry<T> {
    data: T,
    created_at: Instant,
}

/// 模组平台缓存
pub struct ModPlatformCache {
    searches: Arc<RwLock<HashMap<String, CacheEntry<Vec<ModSearchResult>>>>>,
    mod_details: Arc<RwLock<HashMap<String, CacheEntry<ModDetail>>>>,
    versions: Arc<RwLock<HashMap<String, CacheEntry<Vec<ModVersionInfo>>>>>,
    fingerprint_results: Arc<RwLock<HashMap<i64, CacheEntry<Option<FileSource>>>>>,
    config: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub ttl_searches: u64,
    pub ttl_mod_detail: u64,
    pub ttl_versions: u64,
    pub max_entries: usize,
    pub ttl_fingerprint: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl_searches: 300,
            ttl_mod_detail: 600,
            ttl_versions: 600,
            max_entries: 500,
            ttl_fingerprint: 86400,
        }
    }
}

impl ModPlatformCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            searches: Arc::new(RwLock::new(HashMap::new())),
            mod_details: Arc::new(RwLock::new(HashMap::new())),
            versions: Arc::new(RwLock::new(HashMap::new())),
            fingerprint_results: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn get_searches(&self, key: &str) -> Option<Vec<ModSearchResult>> {
        let map = self.searches.read().await;
        map.get(key).filter(|e| e.created_at.elapsed() < Duration::from_secs(self.config.ttl_searches)).map(|e| e.data.clone())
    }

    pub async fn set_searches(&self, key: String, data: Vec<ModSearchResult>) {
        let mut map = self.searches.write().await;
        if map.len() >= self.config.max_entries { map.clear(); }
        map.insert(key, CacheEntry { data, created_at: Instant::now() });
    }

    pub async fn get_mod_detail(&self, key: &str) -> Option<ModDetail> {
        let map = self.mod_details.read().await;
        map.get(key).filter(|e| e.created_at.elapsed() < Duration::from_secs(self.config.ttl_mod_detail)).map(|e| e.data.clone())
    }

    pub async fn set_mod_detail(&self, key: String, data: ModDetail) {
        let mut map = self.mod_details.write().await;
        if map.len() >= self.config.max_entries { map.clear(); }
        map.insert(key, CacheEntry { data, created_at: Instant::now() });
    }

    pub async fn get_versions(&self, key: &str) -> Option<Vec<ModVersionInfo>> {
        let map = self.versions.read().await;
        map.get(key).filter(|e| e.created_at.elapsed() < Duration::from_secs(self.config.ttl_versions)).map(|e| e.data.clone())
    }

    pub async fn set_versions(&self, key: String, data: Vec<ModVersionInfo>) {
        let mut map = self.versions.write().await;
        if map.len() >= self.config.max_entries { map.clear(); }
        map.insert(key, CacheEntry { data, created_at: Instant::now() });
    }

    pub async fn get_fingerprint(&self, fp: i64) -> Option<Option<FileSource>> {
        let map = self.fingerprint_results.read().await;
        map.get(&fp).filter(|e| e.created_at.elapsed() < Duration::from_secs(self.config.ttl_fingerprint)).map(|e| e.data.clone())
    }

    pub async fn set_fingerprint(&self, fp: i64, data: Option<FileSource>) {
        let mut map = self.fingerprint_results.write().await;
        if map.len() >= self.config.max_entries { map.clear(); }
        map.insert(fp, CacheEntry { data, created_at: Instant::now() });
    }
}
```

- [ ] **步骤 2：修改** **`modplatform/mod.rs`** — 注册缓存模块

```rust
pub mod cache;
```

- [ ] **步骤 3：验证编译通过**

```bash
cd McPatch2/manager
cargo check 2>&1
```

- [ ] **步骤 4：Commit**

```bash
git add McPatch2/manager/src/modplatform/cache.rs McPatch2/manager/src/modplatform/mod.rs
git commit -m "feat(modplatform): add caching layer with TTL and LRU eviction"
```

***

### 阶段 4：CurseForge Provider 实现

**文件：**

- 创建：`McPatch2/manager/src/modplatform/curseforge/mod.rs`
- 创建：`McPatch2/manager/src/modplatform/curseforge/client.rs`
- 创建：`McPatch2/manager/src/modplatform/curseforge/search.rs`
- 创建：`McPatch2/manager/src/modplatform/curseforge/versions.rs`
- 创建：`McPatch2/manager/src/modplatform/curseforge/download.rs`
- 创建：`McPatch2/manager/src/modplatform/curseforge/fingerprint.rs`
- 修改：`McPatch2/manager/src/modplatform/mod.rs`
- [ ] **步骤 1：创建** **`curseforge/mod.rs`**

```rust
pub mod client;
pub mod search;
pub mod versions;
pub mod download;
pub mod fingerprint;
```

- [ ] **步骤 2：创建** **`curseforge/client.rs`**

```rust
use std::sync::Arc;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::modplatform::cache::ModPlatformCache;
use crate::modplatform::error::ModPlatformError;
use crate::modplatform::rate_limiter::RateLimiter;
use crate::modplatform::types::*;
use crate::modplatform::ModPlatformProvider;

const CF_BASE_URL: &str = "https://api.curseforge.com/v1";

pub struct CurseForgeProvider {
    client: Client,
    api_key: String,
    cache: Arc<ModPlatformCache>,
    rate_limiter: Arc<RateLimiter>,
}

impl CurseForgeProvider {
    pub fn new(api_key: String, cache: Arc<ModPlatformCache>, rate_limiter: Arc<RateLimiter>) -> Self {
        Self {
            client: Client::builder()
                .user_agent("MCPATCH2/2.1")
                .default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert("x-api-key", api_key.parse().unwrap());
                    headers
                })
                .build()
                .unwrap(),
            api_key,
            cache,
            rate_limiter,
        }
    }

    pub(crate) async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, ModPlatformError> {
        if let Some(wait) = self.rate_limiter.acquire().await {
            tokio::time::sleep(wait).await;
        }

        let url = format!("{}{}", CF_BASE_URL, path);
        let response = self.client.get(&url).send().await
            .map_err(|e| ModPlatformError::NetworkError(e.to_string()))?;

        if response.status().is_rate_limited() {
            let retry_after = response.headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60);
            return Err(ModPlatformError::RateLimited { retry_after_ms: retry_after * 1000 });
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            return Err(ModPlatformError::NetworkError(format!("HTTP {}", status)));
        }

        response.json().await
            .map_err(|e| ModPlatformError::ParseError(e.to_string()))
    }
}

#[async_trait]
impl ModPlatformProvider for CurseForgeProvider {
    fn platform_id(&self) -> PlatformId {
        PlatformId::CurseForge
    }

    async fn search(&self, query: &str, game_version: Option<&str>, mod_loader: Option<&str>) -> Result<Vec<ModSearchResult>, ModPlatformError> {
        search::search(self, query, game_version, mod_loader).await
    }

    async fn get_mod(&self, mod_id: &str) -> Result<ModDetail, ModPlatformError> {
        let cache_key = format!("cf_mod_{}", mod_id);
        if let Some(cached) = self.cache.get_mod_detail(&cache_key).await {
            return Ok(cached);
        }

        let resp: CfModResponse = self.get(&format!("/mods/{}", mod_id)).await?;
        let detail = ModDetail {
            platform: PlatformId::CurseForge,
            mod_id: resp.data.id.to_string(),
            name: resp.data.name,
            slug: resp.data.slug,
            summary: resp.data.summary,
            logo_url: resp.data.logo.url,
            game_versions: resp.data.latest_files.iter().flat_map(|f| f.game_versions.clone()).collect(),
            mod_loaders: resp.data.latest_files.iter().flat_map(|f| f.mod_loaders.iter().map(|l| l.to_string()).collect::<Vec<_>>()).collect(),
            download_count: resp.data.download_count,
        };

        self.cache.set_mod_detail(cache_key, detail.clone()).await;
        Ok(detail)
    }

    async fn get_versions(&self, mod_id: &str, game_version: Option<&str>, mod_loader: Option<&str>) -> Result<Vec<ModVersionInfo>, ModPlatformError> {
        versions::get_versions(self, mod_id, game_version, mod_loader).await
    }

    async fn get_download_url(&self, mod_id: &str, file_id: &str) -> Result<String, ModPlatformError> {
        download::get_download_url(self, mod_id, file_id).await
    }

    async fn batch_resolve_fingerprints(&self, fingerprints: &[i64]) -> Result<Vec<(i64, FileSource)>, ModPlatformError> {
        fingerprint::batch_resolve(self, fingerprints).await
    }
}

// CurseForge API 响应数据结构
#[derive(Deserialize)]
pub(crate) struct CfModResponse {
    pub data: CfModData,
}

#[derive(Deserialize)]
pub(crate) struct CfModData {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub summary: String,
    pub logo: CfLogo,
    pub download_count: i64,
    pub latest_files: Vec<CfFile>,
}

#[derive(Deserialize)]
pub(crate) struct CfLogo {
    pub url: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct CfFile {
    pub id: i64,
    pub display_name: String,
    pub file_name: String,
    pub download_url: String,
    pub game_versions: Vec<String>,
    pub mod_loaders: Vec<CfModLoader>,
}

#[derive(Deserialize, Clone)]
pub(crate) enum CfModLoader {
    #[serde(rename = "forge")]
    Forge,
    #[serde(rename = "fabric")]
    Fabric,
    #[serde(rename = "quilt")]
    Quilt,
    #[serde(rename = "neoforge")]
    NeoForge,
    #[serde(untagged)]
    Other(String),
}

impl std::fmt::Display for CfModLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CfModLoader::Forge => write!(f, "forge"),
            CfModLoader::Fabric => write!(f, "fabric"),
            CfModLoader::Quilt => write!(f, "quilt"),
            CfModLoader::NeoForge => write!(f, "neoforge"),
            CfModLoader::Other(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct CfSearchResponse {
    pub data: Vec<CfSearchItem>,
}

#[derive(Deserialize)]
pub(crate) struct CfSearchItem {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub summary: String,
    pub logo: CfLogo,
    pub latest_files: Vec<CfFile>,
    pub download_count: i64,
}

#[derive(Deserialize)]
pub(crate) struct CfFingerprintResponse {
    pub data: CfFingerprintData,
}

#[derive(Deserialize)]
pub(crate) struct CfFingerprintData {
    pub exact_matches: Vec<CfExactMatch>,
    pub fuzzy_matches: Vec<CfFuzzyMatch>,
}

#[derive(Deserialize)]
pub(crate) struct CfExactMatch {
    pub id: i64,
    pub file: CfFingerprintFile,
}

#[derive(Deserialize)]
pub(crate) struct CfFuzzyMatch {
    pub id: i64,
    pub file: CfFingerprintFile,
}

#[derive(Deserialize)]
pub(crate) struct CfFingerprintFile {
    pub id: i64,
    pub mod_id: i64,
    pub file_name: String,
    pub download_url: String,
}
```

- [ ] **步骤 3：创建** **`curseforge/search.rs`**

```rust
use crate::modplatform::curseforge::client::{CurseForgeProvider, CfSearchItem, CfSearchResponse};
use crate::modplatform::error::ModPlatformError;
use crate::modplatform::types::*;

pub(crate) async fn search(
    provider: &CurseForgeProvider,
    query: &str,
    game_version: Option<&str>,
    mod_loader: Option<&str>,
) -> Result<Vec<ModSearchResult>, ModPlatformError> {
    let cache_key = format!("cf_search_{}_{:?}_{:?}", query, game_version, mod_loader);
    if let Some(cached) = provider.cache.get_searches(&cache_key).await {
        return Ok(cached);
    }

    let mut path = format!("/mods/search?searchFilter={}&pageSize=20", urlencoding(query));
    if let Some(gv) = game_version {
        path.push_str(&format!("&gameVersion={}", gv));
    }
    if let Some(ml) = mod_loader {
        path.push_str(&format!("&modLoaderType={}", ml));
    }

    let resp: CfSearchResponse = provider.get(&path).await?;
    let results: Vec<ModSearchResult> = resp.data.into_iter().map(|item| {
        ModSearchResult {
            platform: PlatformId::CurseForge,
            mod_id: item.id.to_string(),
            name: item.name,
            slug: item.slug,
            summary: item.summary,
            logo_url: item.logo.url,
            game_versions: item.latest_files.iter().flat_map(|f| f.game_versions.clone()).collect(),
            mod_loaders: item.latest_files.iter().flat_map(|f| f.mod_loaders.iter().map(|l| l.to_string()).collect::<Vec<_>>()).collect(),
        }
    }).collect();

    provider.cache.set_searches(cache_key, results.clone()).await;
    Ok(results)
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "%20")
}
```

- [ ] **步骤 4：创建** **`curseforge/versions.rs`**

```rust
use crate::modplatform::curseforge::client::{CurseForgeProvider, CfModResponse};
use crate::modplatform::error::ModPlatformError;
use crate::modplatform::types::*;

pub(crate) async fn get_versions(
    provider: &CurseForgeProvider,
    mod_id: &str,
    game_version: Option<&str>,
    mod_loader: Option<&str>,
) -> Result<Vec<ModVersionInfo>, ModPlatformError> {
    let cache_key = format!("cf_versions_{}_{:?}_{:?}", mod_id, game_version, mod_loader);
    if let Some(cached) = provider.cache.get_versions(&cache_key).await {
        return Ok(cached);
    }

    let mut path = format!("/mods/{}/files?pageSize=50", mod_id);
    if let Some(gv) = game_version {
        path.push_str(&format!("&gameVersion={}", gv));
    }
    if let Some(ml) = mod_loader {
        path.push_str(&format!("&modLoaderType={}", ml));
    }

    type CfFilesResponse = Vec<CfFileItem>;
    let resp: CfFilesResponse = provider.get(&path).await?;

    let versions: Vec<ModVersionInfo> = resp.into_iter().map(|f| {
        ModVersionInfo {
            platform: PlatformId::CurseForge,
            mod_id: mod_id.to_string(),
            file_id: f.id.to_string(),
            name: f.display_name,
            game_version: f.game_versions.first().cloned().unwrap_or_default(),
            mod_loader: f.mod_loaders.first().map(|l| l.to_string()),
            download_url: f.download_url,
            filename: f.file_name,
            sha1_hash: f.hashes.iter().find(|h| h.algo == 1).map(|h| h.value.clone()),
        }
    }).collect();

    provider.cache.set_versions(cache_key, versions.clone()).await;
    Ok(versions)
}

#[derive(serde::Deserialize)]
struct CfFileItem {
    id: i64,
    display_name: String,
    file_name: String,
    download_url: String,
    game_versions: Vec<String>,
    mod_loaders: Vec<super::client::CfModLoader>,
    hashes: Vec<CfHash>,
}

#[derive(serde::Deserialize)]
struct CfHash {
    algo: i64,
    value: String,
}
```

- [ ] **步骤 5：创建** **`curseforge/download.rs`**

```rust
use crate::modplatform::curseforge::client::CurseForgeProvider;
use crate::modplatform::error::ModPlatformError;

pub(crate) async fn get_download_url(
    provider: &CurseForgeProvider,
    mod_id: &str,
    file_id: &str,
) -> Result<String, ModPlatformError> {
    type CfFileResponse = CfFileItem;
    let path = format!("/mods/{}/files/{}", mod_id, file_id);
    let resp: CfFileResponse = provider.get(&path).await?;
    Ok(resp.download_url)
}

#[derive(serde::Deserialize)]
struct CfFileItem {
    download_url: String,
}
```

- [ ] **步骤 6：创建** **`curseforge/fingerprint.rs`**

```rust
use std::collections::HashMap;
use serde::Serialize;
use crate::modplatform::curseforge::client::{CurseForgeProvider, CfFingerprintResponse};
use crate::modplatform::error::ModPlatformError;
use crate::modplatform::types::*;

#[derive(Serialize)]
struct FingerprintRequest {
    fingerprints: Vec<i64>,
}

pub(crate) async fn batch_resolve(
    provider: &CurseForgeProvider,
    fingerprints: &[i64],
) -> Result<Vec<(i64, FileSource)>, ModPlatformError> {
    if fingerprints.is_empty() {
        return Ok(Vec::new());
    }

    // 先查缓存
    let mut results = Vec::new();
    let mut uncached = Vec::new();

    for &fp in fingerprints {
        if let Some(cached) = provider.cache.get_fingerprint(fp).await {
            if let Some(source) = cached {
                results.push((fp, source));
            }
        } else {
            uncached.push(fp);
        }
    }

    if uncached.is_empty() {
        return Ok(results);
    }

    // 限流
    if let Some(wait) = provider.rate_limiter.acquire().await {
        tokio::time::sleep(wait).await;
    }

    let url = format!("{}/fingerprints/fuzzy", crate::modplatform::curseforge::client::CF_BASE_URL);
    let body = FingerprintRequest { fingerprints: uncached.clone() };

    let response = provider.client.post(&url).json(&body).send().await
        .map_err(|e| ModPlatformError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(ModPlatformError::NetworkError(format!("fingerprint API HTTP {}", response.status().as_u16())));
    }

    let resp: CfFingerprintResponse = response.json()
        .await
        .map_err(|e| ModPlatformError::ParseError(e.to_string()))?;

    let mut fp_map: HashMap<i64, FileSource> = HashMap::new();

    for m in resp.data.exact_matches {
        let source = FileSource {
            platform: PlatformId::CurseForge,
            mod_id: m.file.mod_id.to_string(),
            file_id: Some(m.file.id.to_string()),
            download_url: Some(m.file.download_url),
            resolved: true,
        };
        fp_map.insert(m.id, source);
    }

    for m in resp.data.fuzzy_matches {
        let source = FileSource {
            platform: PlatformId::CurseForge,
            mod_id: m.file.mod_id.to_string(),
            file_id: Some(m.file.id.to_string()),
            download_url: Some(m.file.download_url),
            resolved: true,
        };
        fp_map.insert(m.id, source);
    }

    // 填充结果 + 更新缓存
    for &fp in &uncached {
        let source = fp_map.get(&fp).cloned();
        provider.cache.set_fingerprint(fp, source.clone()).await;
        if let Some(s) = source {
            results.push((fp, s));
        }
    }

    Ok(results)
}
```

- [ ] **步骤 7：修改** **`modplatform/mod.rs`** — 注册 curseforge 模块

```rust
pub mod curseforge;
```

- [ ] **步骤 8：验证编译通过**

```bash
cd McPatch2/manager
cargo check 2>&1
```

- [ ] **步骤 9：Commit**

```bash
git add McPatch2/manager/src/modplatform/curseforge/ McPatch2/manager/src/modplatform/mod.rs
git commit -m "feat(modplatform): implement CurseForge provider with fingerprint matching"
```

***

### 阶段 5：Modrinth Provider 实现

**文件：**

- 创建：`McPatch2/manager/src/modplatform/modrinth/mod.rs`
- 创建：`McPatch2/manager/src/modplatform/modrinth/client.rs`
- 创建：`McPatch2/manager/src/modplatform/modrinth/search.rs`
- 创建：`McPatch2/manager/src/modplatform/modrinth/versions.rs`
- 创建：`McPatch2/manager/src/modplatform/modrinth/hash.rs`
- 修改：`McPatch2/manager/src/modplatform/mod.rs`
- [ ] **步骤 1：创建** **`modrinth/mod.rs`**

```rust
pub mod client;
pub mod search;
pub mod versions;
pub mod hash;
```

- [ ] **步骤 2：创建** **`modrinth/client.rs`**

```rust
use std::sync::Arc;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use crate::modplatform::cache::ModPlatformCache;
use crate::modplatform::error::ModPlatformError;
use crate::modplatform::rate_limiter::RateLimiter;
use crate::modplatform::types::*;
use crate::modplatform::ModPlatformProvider;

const MR_BASE_URL: &str = "https://api.modrinth.com/v2";

pub struct ModrinthProvider {
    client: Client,
    cache: Arc<ModPlatformCache>,
    rate_limiter: Arc<RateLimiter>,
}

impl ModrinthProvider {
    pub fn new(cache: Arc<ModPlatformCache>, rate_limiter: Arc<RateLimiter>) -> Self {
        Self {
            client: Client::builder()
                .user_agent("MCPATCH2
```

