# MCPATCH2 多平台模组下载支持 — 功能扩展计划

> **文档版本**: v2.1（深度审查修正版）  
> **项目名称**: MCPATCH2 (MC自动更新系统)  
> **目标**: 集成 CurseForge 与 Modrinth 两大模组分发平台，实现模组资源的智能路由分发——公共模组走平台 CDN，私有模组走服务端分发
> **设计原则**: 管理员放 JAR → 打包 → 玩家零配置零感知
> **关键修正**: 针对 v2.0 的 17 项深度审查发现已全部修正（详见附录 C）

---

## 目录

1. [需求分析与接口调研](#1-需求分析与接口调研)
2. [系统架构调整](#2-系统架构调整)
3. [服务端功能开发](#3-服务端功能开发)
4. [客户端功能开发](#4-客户端功能开发)
5. [测试计划](#5-测试计划)
6. [部署与文档](#6-部署与文档)
7. [风险评估与应对措施](#7-风险评估与应对措施)
8. [里程碑与时间节点](#8-里程碑与时间节点)
9. [附录](#9-附录)

---

## 1. 需求分析与接口调研

### 1.1 CurseForge API 调研

| 项目 | 详情 |
|------|------|
| **文档地址** | [docs.curseforge.com/rest-api](https://docs.curseforge.com/rest-api/) |
| **基础 URL** | `https://api.curseforge.com/v1` |
| **认证方式** | API Key（请求头 `x-api-key`），需在 [CurseForge Console](https://console.curseforge.com/) 注册申请 |
| **限流策略** | 未公开具体阈值，但需注意合理控制请求频率（建议每端点 ≤ 3 req/s，保守估计） |
| **分页限制** | `pageSize` 最大 50，`index + pageSize ≤ 10,000` |
| **Minecraft 游戏 ID** | `432` |
| **Mods 分类 ID** | `9137` |

**核心端点清单：**

| 端点 | 方法 | 用途 | 关键参数 | 本计划中使用场景 |
|------|------|------|----------|-----------------|
| `/mods/search` | GET | 搜索模组 | `gameId`, `classId`, `searchFilter`, `gameVersion`, `modLoaderType`, `sortField` | 管理员 WebUI 搜索 |
| `/mods/{modId}` | GET | 获取模组详情 | `modId` | 搜索结果展示 |
| `/mods/{modId}/files` | GET | 获取文件列表 | `modId`, `gameVersion`, `modLoaderType` | 版本选择 |
| `/mods/{modId}/files/{fileId}/download-url` | GET | 获取下载直链 | `modId`, `fileId` | **pack 阶段获取 CDN URL** |
| `/fingerprints/fuzzy` | POST | **通过数字指纹批量匹配模组** | `fingerprints` 数组（i64，最多 10,000） | **★ pack 阶段核心调用** |
| `/minecraft/version` | GET | 获取支持的 Minecraft 版本列表 | — | 元数据缓存 |

**响应数据关键字段：**

```json
// File 对象（pack 阶段关心的核心字段）
{
  "id": 5000000,
  "displayName": "jei-1.20.1-15.2.0.27.jar",
  "fileName": "jei-1.20.1-15.2.0.27.jar",
  "releaseType": 1,
  "downloadUrl": "https://edge.forgecdn.net/files/...",
  "gameVersions": ["1.20.1", "Forge"],
  "dependencies": [{ "modId": 123, "relationType": 1 }],
  "fileLength": 1234567,
  "hashes": [{ "algo": 1, "value": "sha1hash..." }]
}
```

**加载器枚举值：** `0=Any`, `1=Forge`, `2=Cauldron`, `3=LiteLoader`, `4=Fabric`, `5=Quilt`, `6=NeoForge`

### 1.2 Modrinth API 调研

| 项目 | 详情 |
|------|------|
| **文档地址** | [docs.modrinth.com/api-spec](https://docs.modrinth.com/api-spec) |
| **基础 URL** | `https://api.modrinth.com/v2` |
| **认证方式** | 可选的 API Token（请求头 `Authorization`），搜索等公开接口无需认证 |
| **限流策略** | 未认证：300 req/5min；已认证：900 req/5min |
| **User-Agent 要求** | **必须** 设置 `User-Agent` 头，格式为 `<name>/<version> <contact>` |
| **分页** | 支持 `limit`（默认 10，最大 100）和 `offset` 参数 |

**核心端点清单：**

| 端点 | 方法 | 用途 | 关键参数 | 本计划中使用场景 |
|------|------|------|----------|-----------------|
| `/search` | GET | 搜索项目 | `query`, `facets`, `index`, `offset`, `limit` | 管理员 WebUI 搜索 |
| `/project/{id_or_slug}` | GET | 获取项目详情 | `id`（Base62）或 `slug` | 搜索结果展示 |
| `/project/{id}/version` | GET | 获取版本列表 | `game_versions`, `loaders`, `featured` | 版本选择 |
| `/version_file/{hash}` | GET | **通过 SHA-1 哈希查找版本** | `hash`, `algorithm` | **★ pack 阶段核心调用** |
| `/tag/loader` | GET | 获取加载器列表 | — | 元数据缓存 |
| `/tag/game_version` | GET | 获取游戏版本列表 | — | 元数据缓存 |

**响应数据关键字段：**

```json
// Version 对象（pack 阶段关心的核心字段）
{
  "id": "xyz123",
  "project_id": "AbA9UuXP",
  "name": "Fabric API 0.91.0",
  "version_number": "0.91.0+1.20.1",
  "game_versions": ["1.20.1"],
  "loaders": ["fabric"],
  "files": [{
    "url": "https://cdn.modrinth.com/files/...",
    "filename": "fabric-api-0.91.0+1.20.1.jar",
    "size": 1234567,
    "hashes": { "sha1": "...", "sha512": "..." },
    "primary": true
  }],
  "dependencies": [{ "version_id": "...", "dependency_type": "required" }],
  "release_type": "release"
}
```

### 1.3 平台差异分析与统一抽象层设计

#### 三套哈希体系并存

本项目的关键认知：存在**三套不同的哈希体系**，不可混淆：

| 哈希体系 | 用途 | 算法 | 格式示例 | 使用方 |
|----------|------|------|----------|--------|
| **MCPATCH2 内部哈希** | 文件完整性校验 | CRC64_XZ + CRC16_IBM_SDLC | `82e09fc553b335ab_1306` | 服务端 Diff 对比 + 客户端文件校验 |
| **CurseForge 指纹** | 模组身份识别 | 自定义 MurmurHash2 64位 | `1234567890`（i64 十进制） | pack 阶段 /fingerprints/fuzzy 匹配 |
| **Modrinth 哈希** | 模组版本查找 | SHA-1 | `a1b2c3d4e5f6...`（40 hex 字符） | pack 阶段 /version_file/{hash} 查询 |

**关键约束：**
- CurseForge `/fingerprints/fuzzy` 期望**64位数字**（不是 SHA-1 字符串），指纹算法是 CurseForge 自定义的 MurmurHash2 变体，需要额外实现
- Modrinth 正确使用标准 SHA-1 哈希
- MCPATCH2 现有文件校验系统使用 CRC64+CRC16，**不改动**

#### 统一抽象层关键设计决策

1. **pack 阶段识别≠运行时识别**：模组平台匹配仅发生在服务端 `pack` 命令执行时，客户端不做任何平台 API 调用
2. **优先 CurseForge 指纹匹配**：CF 支持批量指纹匹配（单次调用最多 10,000 个），效率远高于 Modrinth 单文件哈希查询，故先查 CF，未命中再查 MR
3. **双重哈希策略**：pack 时对每个模组文件同时计算 CF 指纹（用于 CF 匹配）+ SHA-1（用于 MR 匹配），MCPATCH2 内部哈希（CRC64+CRC16）保持不变
4. **双重下载保障**：即使模组匹配到平台 URL，服务端仍将文件保留在更新包中，客户端 CDN 下载失败时自动降级到服务端

| 对比维度 | CurseForge | Modrinth |
|----------|-----------|----------|
| **模组标识** | 整数 ID（如 `238222`） | Base62 字符串 ID（如 `AbA9UuXP`）或 Slug |
| **指纹匹配** | `POST /fingerprints/fuzzy`，批量（最多 10,000），输入 i64 数组 | `GET /version_file/{hash}`，单文件 SHA-1 查询 |
| **下载链接** | `downloadUrl` 字段或调用 `/download-url` 端点 | `files[].url` 字段直接提供 CDN URL |
| **指纹算法** | 自定义 MurmurHash2 变体（64 位整数） | SHA-1 / SHA-512 |
| **版本号风格** | 文件名中嵌入版本，如 `jei-1.20.1-15.2.0.27.jar` | 独立 `version_number` 字段，如 `0.91.0+1.20.1` |
| **依赖声明** | `dependencies[].relationType`（1=嵌入库, 2=可选, 3=必需） | `dependencies[].dependency_type`（required, optional, incompatible, embedded） |
| **API 认证** | **必需** API Key | 哈希查询无需认证 |

---

## 2. 系统架构调整

### 2.1 核心流程总览

```
┌─────────────────────────────────────────────────────────────────────┐
│                   智能路由分发核心流程                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  管理员操作:                                                         │
│    ┌────────────────────────────────────────────┐                   │
│    │ 将模组 JAR 放入 workspace/mods/              │                   │
│    └────────────────┬───────────────────────────┘                   │
│                     │                                                │
│                     ▼                                                │
│    ┌────────────────────────────────────────────────────────────┐   │
│    │ 执行 pack 命令（或 pack --preview 预览模式）                 │   │
│    │                                                            │   │
│    │  for each .jar in workspace/mods/:                          │   │
│    │    ① 计算 CF 指纹 (MurmurHash2 64位) + SHA-1               │   │
│    │    ② CurseForge 指纹匹配 (批量)  ──匹配成功──► 记录 CF 信息  │   │
│    │    ③ Modrinth SHA-1 哈希查询    ──匹配成功──► 记录 MR 信息  │   │
│    │    ④ 均未命中 → 标记为"私有模组"                            │   │
│    │    ⑤ pack --preview 输出路由预览表，等待管理员确认            │   │
│    │    ⑥ 确认后生成更新元数据 (含 source 路由字段)               │   │
│    └────────────────┬───────────────────────────────────────────┘   │
│                     │                                                │
│                     ▼                                                │
│    ┌────────────────────────────────────────────────────────────┐   │
│    │ 更新包元数据示例:                                           │   │
│    │ [                                                         │   │
│    │   { path:"mods/jei.jar",   source:"cf",    url:"...cdn..."},│   │
│    │   { path:"mods/fapi.jar",  source:"mr",    url:"...cdn..."},│   │
│    │   { path:"mods/secret.jar",source:"server", hash:"crc64..."}│   │
│    │ ]                                                          │   │
│    └────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  玩家更新流程:                                                       │
│    ┌────────────────────────────────────────────┐                   │
│    │ 收到更新命令 → 下载元数据                     │                   │
│    │ 遍历 changes:                               │                   │
│    │   source=server/null → 走 MCPATCH2 协议下载  │                   │
│    │   source=cf/mr      → 从 CDN URL 下载        │                   │
│    │                      → CDN 失败 → fallback   │                   │
│    │   全部校验 CRC64+CRC16 内部哈希               │                   │
│    └────────────────────────────────────────────┘                   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Rust 服务端：平台无关 Provider 接口

在 `McPatch2/manager/src/` 下新增 `modplatform/` 模块：

```rust
// modplatform/types.rs — 统一数据模型

/// 模组平台资源标识
pub enum PlatformId {
    CurseForge(i32),           // CurseForge 使用整数 ID
    Modrinth(String),          // Modrinth 使用 Base62 字符串 ID
}

/// 更新包文件来源
pub enum FileSource {
    Server,                    // 服务端直接分发
    CurseForge {               // 从 CurseForge CDN 下载
        mod_id: i32,
        file_id: i32,
    },
    Modrinth {                 // 从 Modrinth CDN 下载
        project_id: String,
        version_id: String,
    },
}

/// 更新包中的文件条目（扩展元数据）
/// 注意: 此结构体仅用于 pack 阶段的 ModRouter 输出，
/// 最终写入 VersionMeta 时只保留 source + download_url + mcpatch_hash 三个字段
pub struct UpdateFileEntry {
    pub path: String,
    pub source: FileSource,
    pub size: u64,
    pub mcpatch_hash: String,       // MCPATCH2 内部 CRC64+CRC16 哈希
    pub sha1: Option<String>,        // SHA-1 哈希（仅用于 Modrinth 查询）
    pub cf_fingerprint: Option<i64>, // CF 数字指纹（仅用于 CurseForge 查询）
    pub download_url: Option<String>, // CDN URL，source=Server 时为 None
    pub dependencies: Vec<ModDependency>,
}

/// 统一模组搜索结果（仅供 WebUI 搜索使用）
pub struct ModSearchResult {
    pub platform: PlatformKind,
    pub id: PlatformId,
    pub slug: String,
    pub name: String,
    pub summary: String,
    pub icon_url: Option<String>,
    pub authors: Vec<String>,
    pub download_count: u64,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
}

/// CurseForge 指纹匹配请求（pack 阶段使用）
pub struct FingerprintRequest {
    pub fingerprints: Vec<i64>,       // CurseForge 64位数字指纹（非 SHA-1）
}

/// 指纹匹配结果
pub struct FingerprintMatch {
    pub cf_fingerprint: i64,          // 输入的 CF 数字指纹
    pub platform: PlatformKind,
    pub mod_id: PlatformId,
    pub file_id: String,
    pub download_url: String,
    pub filename: String,
    pub size: u64,
}

/// 平台 Provider trait
#[async_trait]
pub trait ModPlatformProvider: Send + Sync {
    /// 搜索模组（WebUI 管理用）
    async fn search(&self, query: &SearchQuery) -> Result<Vec<ModSearchResult>, PlatformError>;

    /// 获取模组详情（WebUI 管理用）
    async fn get_mod(&self, id: &PlatformId) -> Result<ModSearchResult, PlatformError>;

    /// 获取版本列表（WebUI 管理用）
    async fn get_versions(&self, id: &PlatformId, filter: &VersionFilter) -> Result<Vec<ModVersionInfo>, PlatformError>;

    /// 获取下载 URL（WebUI 管理用 + pack 阶段刷新 URL）
    async fn get_download_url(&self, file_id: &str, mod_id: &PlatformId) -> Result<String, PlatformError>;

    /// 批量指纹匹配（★ pack 阶段核心方法 — CurseForge）
    async fn batch_resolve_fingerprints(&self, request: &FingerprintRequest) -> Result<Vec<FingerprintMatch>, PlatformError>;

    /// 单文件哈希查找（★ pack 阶段核心方法 — Modrinth）
    async fn resolve_hash(&self, hash: &str, algo: HashAlgorithm) -> Result<Option<FingerprintMatch>, PlatformError>;
}
```

**设计说明：** 与 v2.0 相比，`FingerprintRequest.fingerprints` 类型从 `Vec<String>` 修正为 `Vec<i64>`（CurseForge 使用 64 位数字指纹）。`UpdateFileEntry` 增加了 `sha1` 和 `cf_fingerprint` 临时字段用于 pack 阶段的识别过程，最终写入元数据时只保留 `mcpatch_hash`（即现有的 CRC64+CRC16 格式）。

### 2.3 Pack 阶段模组识别引擎

这是整个方案的核心新增模块，位于 `task/mod_router.rs`：

```rust
// task/mod_router.rs — Pack 阶段模组识别与路由决策

pub struct ModRouter {
    cf_provider: Arc<CurseForgeProvider>,
    mr_provider: Arc<ModrinthProvider>,
    config: ModPlatformConfig,
    fingerprint_cache: Arc<Mutex<HashMap<String, FileSource>>>,  // 增量 pack 缓存
}

impl ModRouter {
    /// 核心方法：对 workspace/mods/ 目录下的所有 JAR 进行识别和路由
    pub async fn resolve_mods(&self, mods_dir: &Path, changed_files: &[String]) -> Result<Vec<UpdateFileEntry>> {
        // 阶段一：收集需要检测的 JAR 文件
        // 增量 pack 时仅检测 changed_files，已有缓存结果的文件跳过
        let jars = self.collect_changed_jars(mods_dir, changed_files)?;

        // 阶段二：双重哈希计算
        // 对每个 JAR 文件同时计算 CF 指纹 (i64) + SHA-1 + 内部 CRC64+CRC16
        let hashes: Vec<(PathBuf, i64, String, String)> = jars
            .iter()
            .map(|path| {
                let cf_fp = compute_cf_fingerprint(path);  // MurmurHash2 变体
                let sha1 = compute_sha1(path);              // 标准 SHA-1
                let mc_hash = compute_mcpatch_hash(path);   // CRC64+CRC16（现有逻辑）
                (path.clone(), cf_fp, sha1, mc_hash)
            })
            .collect();

        // 阶段三：优先批量匹配 CurseForge（最多 10,000 个指纹）
        let cf_fingerprints: Vec<i64> = hashes.iter().map(|(_, fp, _, _)| *fp).collect();
        let cf_matches = self.cf_provider
            .batch_resolve_fingerprints(&FingerprintRequest {
                fingerprints: cf_fingerprints,
            }).await?;
        let cf_matched: HashSet<i64> = cf_matches.iter().map(|m| m.cf_fingerprint).collect();

        // 阶段四：未命中的逐个查询 Modrinth（使用 SHA-1）
        let mut mr_matches = Vec::new();
        for (path, cf_fp, sha1, mc_hash) in &hashes {
            if cf_matched.contains(cf_fp) { continue; }
            if let Some(m) = self.mr_provider.resolve_hash(sha1, HashAlgorithm::Sha1).await? {
                mr_matches.push((path.clone(), m, mc_hash.clone()));
            }
        }

        // 阶段五：生成路由结果 + 更新缓存
        let entries = self.build_routing_entries(hashes, cf_matches, mr_matches);

        // 更新增量缓存
        self.update_fingerprint_cache(&entries);

        Ok(entries)
    }

    /// 将路由结果合并到 VersionMeta 中
    /// 仅在 update-file 类型的变更中附加 source + download_url 字段
    pub fn merge_into_version_meta(
        &self,
        meta: &mut VersionMeta,
        mod_entries: &[UpdateFileEntry],
    );
}
```

### 2.4 客户端数据模型变更

Java 客户端仅需在现有 `FileChange.UpdateFile` 中增加两个字段：

```java
// FileChange.java — 在现有 UpdateFile 内部类中增加

public class UpdateFile implements FileChange {
    // 现有字段（保留不变）
    public String path;
    public String hash;       // CRC64+CRC16 格式，保持向后兼容
    public long len;
    public long modified;
    public long offset;

    // 新增字段（可选，向下兼容）
    // source=null 或 "server" 时走传统服务端下载
    // source="curseforge" 或 "modrinth" 时走 CDN 下载
    public String source;
    public String downloadUrl;
}
```

**JSON 兼容性说明：**
- 旧版更新包（无 `source` / `downloadUrl` 字段）→ 客户端按 `source=null` 处理，走服务端下载
- 新版更新包 + 旧版客户端 → 旧版客户端会忽略未知字段，同样走服务端下载（宽容解析）
- 序列化时 `source=null` 字段不写入 JSON，保持旧版格式一致

客户端 **不需要** 新增任何数据类。所有平台相关的复杂类型（`ModMeta`、`ModVersionInfo`、`ModDependency`、`FingerprintMatch` 等）仅存在于服务端。

客户端 **不需要** 新增任何网络能力。CDN 下载可**直接复用**现有的 `HttpProtocol`，该协议已支持 HTTP/HTTPS、自定义 Headers、超时配置和 SSL 证书控制。双源降级可**复用**现有的 `Servers.multipleAvailableServers()` 自动重试/切换机制。

### 2.5 配置文件扩展

#### 服务端 `config.toml` 新增段

```toml
[modplatform]
enabled = true
# 模组目录（相对于 workspace），支持数组
mod-dirs = ["mods"]
# 模组文件扩展名
mod-extensions = [".jar", ".zip"]

[modplatform.curseforge]
api-key = "YOUR_CURSEFORGE_API_KEY"

[modplatform.modrinth]
# 可选，提供 token 可获得更高频率限制
api-token = ""

[modplatform.cache]
ttl-searches = 300          # 搜索结果缓存秒数
ttl-mod-detail = 1800       # 模组详情缓存秒数
ttl-versions = 600          # 版本列表缓存秒数
max-entries = 1000          # 最大缓存条目数
# 增量 pack 时指纹缓存保留时间
ttl-fingerprint-cache = 86400  # 24 小时

[modplatform.proxy]
# 可选：服务端访问平台 API 时使用的 HTTP 代理
enabled = false
host = ""
port = 0
```

#### 客户端 `mcpatch.yml` 无需任何变更

客户端配置 **不需要** 增加任何模组平台相关字段。即使管理员希望客户端通过代理下载 CDN 资源，也建议通过系统级代理或 `JVM` 参数（`-Dhttps.proxyHost`）解决，保持配置文件的零变更。

### 2.6 版本比较机制（仅服务端 WebUI 使用）

版本比较仅在服务端管理员搜索模组、选择版本时使用，不是客户端逻辑：

```
ModPlatformVersionComparator
├── CurseForgeComparator
│   └── 逻辑：解析文件名中的版本号段，结合 releaseType 排序
│        Release > Beta > Alpha
│        同一类型下按 semantic version 比较
│
└── ModrinthComparator
    └── 逻辑：直接使用 version_number 字段
        优先按 featured 标记筛选
        再按 semantic version 比较
        回退到 date_published 字段
```

---

## 3. 服务端功能开发

### 3.1 CurseForge Provider 实现

| 任务 | 详细描述 | 交付物 |
|------|---------|--------|
| **API 客户端封装** | 使用 `reqwest` 封装 HTTP 请求，处理 API Key 鉴权、限流器集成 | `modplatform/curseforge/client.rs` |
| **搜索功能** | 实现模组搜索，支持按名称、Minecraft 版本、加载器类型过滤 | `modplatform/curseforge/search.rs` |
| **版本列表获取** | 获取指定模组的所有版本，支持按 Minecraft 版本和加载器过滤 | `modplatform/curseforge/versions.rs` |
| **下载 URL 获取** | 从文件信息中提取或请求下载直链 | `modplatform/curseforge/download.rs` |
| **★ 批量指纹匹配** | 调用 `POST /fingerprints/fuzzy` 批量匹配模组身份，**pack 阶段核心** | `modplatform/curseforge/fingerprint.rs` |

**批量指纹匹配实现要点：**

```rust
// fingerprint.rs — 关键实现逻辑
pub async fn batch_resolve_fingerprints(
    &self,
    request: &FingerprintRequest,
) -> Result<Vec<FingerprintMatch>> {
    // CurseForge 限制单次最多 10,000 个指纹
    const BATCH_SIZE: usize = 10000;
    let mut results = Vec::new();

    for chunk in request.fingerprints.chunks(BATCH_SIZE) {
        let resp = self.client
            .post("/fingerprints/fuzzy")
            .json(&serde_json::json!({
                "fingerprints": chunk   // 注意：类型为 [i64]，非字符串
            }))
            .send()
            .await?;

        // 解析响应中的 fuzzyMatches
        // 每个 fuzzyMatch 包含: modId, fileId, fingerprint(输入的i64), latestFiles[]
        for fuzzy_match in resp.fuzzy_matches {
            // 取第一个 exact match 的文件信息
            if let Some(file) = fuzzy_match.latestFiles.first() {
                results.push(FingerprintMatch {
                    cf_fingerprint: fuzzy_match.fingerprint,
                    platform: PlatformKind::CurseForge,
                    mod_id: PlatformId::CurseForge(fuzzy_match.mod_id as i32),
                    file_id: file.id.to_string(),
                    download_url: file.download_url.clone(),
                    filename: file.file_name.clone(),
                    size: file.file_length as u64,
                });
            }
        }
    }
    Ok(results)
}
```

### 3.2 Modrinth Provider 实现

| 任务 | 详细描述 | 交付物 |
|------|---------|--------|
| **API 客户端封装** | 使用 `reqwest` 封装，支持可选的 Token 认证、限流器集成 | `modplatform/modrinth/client.rs` |
| **搜索功能** | 使用 facets 语法支持多维筛选 | `modplatform/modrinth/search.rs` |
| **版本列表获取** | 支持 `game_versions` 和 `loaders` 数组过滤 | `modplatform/modrinth/versions.rs` |
| **★ 哈希查找** | 通过 SHA-1 哈希值定位版本，**pack 阶段兜底查询** | `modplatform/modrinth/hash.rs` |

### 3.3 元数据缓存机制

由于 CurseForge API 需要 API Key 且有频率限制，必须实现缓存层：

```
CacheLayer
├── 内存缓存 (HashMap + TTL + RwLock)
│   ├── 缓存粒度：模组搜索结果、模组详情、版本列表
│   ├── TTL 策略：搜索结果 5 分钟、模组详情 30 分钟、版本列表 10 分钟
│   ├── ★ 指纹匹配结果：增量 pack 时缓存 24 小时（ttl-fingerprint-cache）
│   │   缓存 key = 文件路径 + 文件修改时间 + 文件大小，三者任一变化则缓存失效
│   └── ★ 并发安全：使用 RwLock 而非 Mutex，读多写少场景下性能更高
│
├── 磁盘缓存 (JSON 文件，可选)
│   └── 缓存粒度：模组元数据（长期保留，服务重启后不丢失）
│   └── 存储位置：`{working_dir}/modplatform_cache/{platform}/`
│   └── 失效策略：通过 `date_modified` 字段判断，超过 7 天淘汰
│
└── 缓存管理器配置项
    └── `cache-ttl-searches: 300`          // 搜索结果缓存秒数
    └── `cache-ttl-mod-detail: 1800`       // 模组详情缓存秒数
    └── `cache-ttl-versions: 600`          // 版本列表缓存秒数
    └── `cache-ttl-fingerprint-cache: 86400` // 指纹缓存秒数
    └── `cache-max-entries: 1000`          // 最大缓存条目数
```

### 3.4 REST API 扩展（管理员 WebUI 用）

| 路由 | 方法 | 功能 | 认证 |
|------|------|------|------|
| `/api/modplatform/search` | POST | 跨平台模组搜索 | 需认证 |
| `/api/modplatform/mod/{platform}/{id}` | GET | 获取模组详情 | 需认证 |
| `/api/modplatform/mod/{platform}/{id}/versions` | GET | 获取模组版本列表 | 需认证 |
| `/api/modplatform/download` | POST | 获取文件下载链接 | 需认证 |
| `/api/modplatform/pack/preview` | POST | **预览 pack 的路由结果（不实际打包）** | 需认证 |

**新增 `/api/modplatform/pack/preview` 端点：**

```json
// POST /api/modplatform/pack/preview
// Request: 空（自动扫描当前 workspace）
// Response:
{
  "code": 1,
  "data": {
    "total_files": 15,
    "routing": {
      "curseforge": 8,
      "modrinth": 5,
      "server": 2
    },
    "estimated_bandwidth_saving": "85%",
    "details": [
      {
        "filename": "jei-1.20.1-15.2.0.27.jar",
        "source": "curseforge",
        "matched": true,
        "mod_name": "Just Enough Items (JEI)",
        "mod_id": 238222,
        "size": 1234567
      },
      {
        "filename": "secret-mod.jar",
        "source": "server",
        "matched": false,
        "reason": "未在任何平台找到匹配",
        "size": 54321,
        "will_be_sent_by_server": true
      }
    ]
  }
}
```

### 3.5 Pack 阶段模组识别与路由逻辑（核心）

修改 `task/pack.rs`，在现有 Diff 逻辑中插入模组识别步骤：

```
现有 pack 流程：
  workspace/ 对比 history/ → 生成增量更新包

扩展后的 pack 流程：
  ① 执行常规 Diff，获取变更文件列表

  ② 从变更文件中筛选出 mods/ 目录下的 .jar 文件
     默认模组目录：config.toml 中 `modplatform.mod-dirs`（默认 ["mods"]）
     默认模组扩展名：config.toml 中 `modplatform.mod-extensions`（默认 [".jar", ".zip"]）

  ③ 计算双重哈希：CF 指纹 (i64) + SHA-1 + MCPATCH2 内部哈希

  ④ 执行批量指纹识别（ModRouter.resolve_mods）
     a. 先查增量缓存（文件路径+修改时间+大小未变则跳过识别）
     b. 批量发送到 CurseForge /fingerprints/fuzzy
     c. 未命中的逐个查询 Modrinth /version_file/{hash}
     d. 生成路由决策（cf / mr / server）

  ⑤ pack --preview 模式：输出路由预览表后终止，不实际打包

  ⑥ 将路由结果合并到更新包元数据
     匹配到的模组：记录 source + download_url + 预期 MCPATCH2 哈希
     未匹配的模组：标记 source=server，文件保留在更新包中

  ⑦ 常规文件处理不变
```

**非 JAR 模组文件的处理策略：**

| 类型 | 识别方式 | 路由决策 |
|------|---------|---------|
| `.jar` 文件 | CF 指纹 + MR SHA-1（完整识别） | CF/MR 优先，未命中走服务端 |
| `.zip` 文件 | 仅尝试 Modrinth SHA-1 查询 | 命中走 CDN，未命中走服务端 |
| 其他文件 | 不参与模组识别 | 走现有服务端分发逻辑 |

### 3.6 CDN URL 刷新与过期处理

平台 CDN 下载链接可能有时效性（尤其 CurseForge 的 `downloadUrl` 字段），需要过期处理机制：

```
URL 过期处理策略：

  策略A（推荐）— 客户端双源降级：
    更新包中始终保留模组文件的副本
    客户端下载顺序：CDN URL → 失败 → 服务端下载
    优点：客户端无额外 API 调用，完全零配置
    缺点：服务端仍占用存储空间

  策略B — 服务端按需刷新：
    客户端 CDN 下载失败 → 通知服务端 → 服务端调用平台 API 刷新 URL
    将新 URL 返回给客户端重试
    优点：节省服务端存储
    缺点：需要客户端-服务端额外通信

  策略C — Pack 时验证 URL 有效性：
    pack 时获取 URL 后立即 HEAD 请求验证
    对无效 URL 直接降级为 source=server
    优点：从源头避免过期 URL
    缺点：增加 pack 时间
```

**推荐组合策略：** 默认采用 **策略A**，pack 时额外执行 **策略C** 作为预防性检查。对于对带宽敏感的部署场景，可启用 **策略B** 作为补充。

### 3.7 离线打包模式与错误恢复

当管理员在有网络限制的环境中打包时，系统必须优雅降级：

```
离线模式处理策略：

  [config.toml] 新增选项：
  [modplatform]
  offline-mode = false   # 手动开启后跳过所有 API 调用

  自动降级（offline-mode = false 时）:
  ├── CF / MR API 全部超时(>10s)
  │   └── 自动将所有模组标记为 source=server
  │   └── 输出警告日志: "平台 API 不可用，所有模组降级为服务端分发"
  │
  ├── 部分失败（CF 成功，MR 超时）
  │   └── CF 匹配结果正常使用
  │   └── MR 未查询到的标记为 server
  │
  └── Pack 中途网络中断
      └── 已写入的部分结果不保留（原子性失败）
      └── 重新 pack 时从头开始

  增量 pack 的缓存保护:
  ├── 已缓存的路由结果不会因临时网络问题失效
  ├── 仅新文件/变化文件需要重新识别
```

### 3.8 并发安全与运维保障

```
并发安全：
├── CacheLayer 使用 RwLock（读多写少）
├── 同一 workspace 不允许并发 pack（文件锁控制）
│   └── 使用 `{workspace_dir}/.pack-lock` 文件作为互斥锁
├── 限流器使用原子计数器 + 定时重置（令牌桶）
│   └── CF ≤ 3 req/s，MR ≤ 5 req/s

日志安全：
├── API Key / Token 必须脱敏
│   └── 日志输出时替换为 "****{last4}"
│   └── 异常消息中自动过滤
├── 不记录完整的 download_url（可能包含临时鉴权参数）
└── 错误堆栈中移除敏感 headers

可运维性命令（CLI）:
├── mcpatch check-api
│   └── 验证 CF / MR API 连通性
│   └── 输出: "CF API OK (0.23s)" / "MR API FAILED: 401 Unauthorized"
│
├── mcpatch pack --preview
│   └── 输出路由预览表（已在 3.4 定义）
│
└── mcpatch cache clear [platform]
    └── 手动清除模组平台缓存
```

---

## 4. 客户端功能开发

### 4.1 更新元数据解析扩展

玩家客户端几乎不需要新增模块，仅需在现有 `Work.java` 的更新流程中增加对 `source` 字段的处理：

```java
// Work.java — 下载流程扩展点

// 现有流程：遍历 changes → 遇到 UpdateFile → 下载
// 扩展后：判断 source 字段决定下载来源

void processUpdateFile(UpdateFile file) {
    if (file.source == null || "server".equals(file.source)) {
        // 传统路径：从 MCPATCH2 服务端下载（复用现有逻辑）
        downloadFromServer(file);
    } else {
        // CDN 路径：从平台 CDN 下载（复用 HttpProtocol）
        // 失败时回退到 downloadFromServer
        downloadFromCdnWithFallback(file);
    }
}

void downloadFromCdnWithFallback(UpdateFile file) {
    try {
        // 复用已有的 HttpProtocol 进行 CDN 下载
        // HttpProtocol 已支持: https、自定义 headers、超时配置
        httpDownload(file.downloadUrl, file.tempPath);
        // 校验使用现有的 CRC64+CRC16 哈希
        verifyHash(file.tempPath, file.hash);
    } catch (Exception e) {
        logger.warn("CDN 下载失败，回退到服务端: {}", file.path);
        downloadFromServer(file);
    }
}
```

**关键说明：** 客户端**不需要**新增 HTTP 客户端。现有代码已在 `Servers.java` 中通过 `HttpProtocol` 实现了完整的 HTTP(S) 下载能力，CDN 下载直接复用即可。双源降级逻辑可封装在 `Servers.java` 的 `multipleAvailableServers()` 框架中，复用现有的重试/切换机制。

### 4.2 校验机制

沿用现有的校验能力，不对所有来源区别对待：

```
所有下载完成的文件统一执行：
  ① 大小校验：与元数据中的 len 对比
  ② CRC64+CRC16 哈希校验：复用 HashUtility.calculateHash()，与元数据 hash 对比
```

**注意：** MCPATCH2 内部使用 CRC64_XZ + CRC16_IBM_SDLC 作为文件校验算法，**不是** SHA-1。现有 `HashUtility.calculateHash()` 方法正确实现了此算法，无需修改。

### 4.3 用户界面变更

玩家客户端 UI **几乎不需要变更**。唯一可选的增强：

| UI 组件 | 功能 | 触发场景 |
|---------|------|----------|
| **下载来源指示器** | 在更新进度中显示文件来源（服务端/CDN） | 更新进行中 |
| **CDN 下载失败提示** | 告知玩家"正在从备用源下载" | CDN 失败回退时 |

以上均为 **可选增强**，不影响核心功能。最小可行版本可完全不做 UI 变更。

### 4.4 错误处理策略

| 错误类型 | 处理方式 | 用户提示 |
|----------|---------|----------|
| CDN 下载失败 | 自动回退到服务端下载，重试 2 次 | "正在从备用源下载 {filename}..." |
| CDN + 服务端均失败 | 跳过该文件，标记失败 | "下载失败：{filename}，请稍后重试" |
| CRC64+CRC16 校验失败 | 重新下载，最多重试 2 次 | "文件校验失败，正在重新下载 (2/3)" |
| CDN URL 404 | 立即降级到服务端，不重试 CDN | "下载地址已过期，正在从备用源获取" |

---

## 5. 测试计划

### 5.1 单元测试

| 测试模块 | 测试项 | 预期结果 |
|---------|--------|---------|
| **CurseForge Provider** | 单文件指纹匹配 | 正确调用 `/fingerprints/fuzzy`，请求体为 `[i64]` 数组 |
| | 批量指纹匹配（1 个 / 100 个 / 10,000 个 / 12,000 个分两批） | 正确分块，合并结果 |
| | 批量指纹回包解析 | 正确映射 `fuzzyMatches` → `FingerprintMatch` |
| | API Key 鉴权头设置 | 请求头包含 `x-api-key` |
| | URL 过期后重新获取 | 调用 `/download-url` 返回新 URL |
| | **API Key 错误 (401)** | 返回明确的 `PlatformError::AuthFailed` 错误 |
| | **无网络时调用** | 超时后返回 `PlatformError::NetworkError` |
| **Modrinth Provider** | 单文件 SHA-1 哈希查询 | 正确调用 `/version_file/{hash}?algorithm=sha1` |
| | SHA-1/SHA-512 算法参数 | 请求参数中的算法值正确 |
| | User-Agent 头设置 | 请求包含合规的 User-Agent |
| | 哈希未命中处理 | 返回 `None`，不报错 |
| **ModRouter** | 全 CF 匹配的场景 | 所有文件走 CDN 路由 |
| | 全未命中的场景 | 所有文件走服务端路由 |
| | 混合场景（CF 匹配 + MR 匹配 + 未命中） | 路由决策正确 |
| | 非 JAR 文件过滤 | .txt/.png/.yml 等不参与识别 |
| | 空目录处理 | 无文件时返回空列表 |
| | **增量 pack 缓存命中** | 未变化的文件跳过 API 调用，直接返回缓存结果 |
| | **增量 pack 缓存失效** | 文件修改时间/大小变化时重新识别 |
| | **离线模式 (offline-mode)** | 跳过所有 API 调用，全部标记为 server |
| | **CF 指纹计算算法** | 验证 MurmurHash2 变体与 CurseForge 官方结果一致 |
| **缓存层** | TTL 过期逻辑 | 过期条目被自动淘汰 |
| | 缓存击穿保护 | 并发请求下只有一个穿透到 API |
| | 磁盘缓存持久化 | 重启后仍可读取 |
| | **并发读写安全** | 多线程同时访问不 panic |
| **URL 过期处理** | 策略 A：客户端双源降级 | CDN 失败后从服务端下载成功 |
| | 策略 C：pack 时 HEAD 验证 | 无效 URL 降级为 server |
| **并发安全** | **同一 workspace 并发 pack** | 文件锁阻止第二个 pack，返回明确错误 |
| | **限流器并发** | 多线程下请求频率不超过配置阈值 |

### 5.2 集成测试

| 场景 | 测试步骤 | 验证点 |
|------|---------|--------|
| **纯 CF 模组包** | 放 5 个 CF 可识别的 JAR → pack → 客户端更新 | 所有文件 CDN 下载，CRC64+CRC16 哈希匹配 |
| **纯私有模组包** | 放 5 个自定义 JAR → pack → 客户端更新 | 所有文件服务端下载，CRC64+CRC16 哈希匹配 |
| **混合包** | 3 个 CF + 2 个 MR + 1 个私有 → pack → 更新 | 路由正确，各有各的来源 |
| **CDN 失效** | pack 后手动破坏 CDN URL → 客户端更新 | 自动降级到服务端下载 |
| **增量更新** | 第一次 pack → 替换 1 个 JAR → 第二次 pack → 更新 | 仅变化的模组文件重新识别 |
| **旧版更新包** | 去掉 source 字段的旧版 pack → 客户端更新 | 走了服务端下载，功能正常 |
| **旧版客户端** | 新版 pack → 旧版客户端 | 忽略 source 字段，走服务端下载 |
| **离线 pack** | 断网后 pack → 输出 | 所有模组标记为 server，输出警告 |
| **错误 API Key** | 配置错误的 CF API Key → pack | pack 失败，明确提示 API Key 无效 |
| **超大模组包** | 12,000 个 JAR → pack | 分两批发送 CF 指纹，正确处理 |

### 5.3 端到端测试

```
场景 A: 标准整合包发布流程
  1. 管理员从 CurseForge 下载 JEI 放入 workspace/mods/
  2. 从 Modrinth 下载 Fabric API 放入 workspace/mods/
  3. 放入 1 个自研私有模组
  4. 执行 pack --preview 验证路由结果
  5. 确认后执行 pack 命令
  6. 验证 pack 输出：JEI=CF路由, Fabric API=MR路由, 私有模组=服务端路由
  7. 客户端执行更新
  8. 验证 JEI 和 Fabric API 从 CDN 下载
  9. 验证私有模组从服务端下载
  10. 验证所有文件 CRC64+CRC16 哈希匹配

场景 B: 模组替换
  1. 第一次 pack → 客户端更新成功
  2. 管理员将 JEI 替换为新版本
  3. 第二次 pack（增量识别，仅 JEI 重新查询 API）
  4. 客户端增量更新
  5. 验证：旧版 JEI 被替换，未变化的模组（Fabric API）不重新下载

场景 C: CDN 全链路故障
  1. pack 完成（所有模组有 CDN URL）
  2. 手动断开客户端与 CDN 的网络连接
  3. 客户端执行更新
  4. 验证：所有模组文件从服务端成功下载，CRC64+CRC16 校验通过

场景 D: 回滚测试
  1. 部署新版 pack（含模组路由）
  2. 客户端更新成功
  3. 管理员回滚到旧版（不包含路由字段）
  4. 客户端再次更新
  5. 验证：客户端正确处理无 source 字段的旧版元数据
```

### 5.4 兼容性测试

| 测试项 | 说明 |
|--------|------|
| **旧版更新包兼容** | 不包含 `source` 字段的旧版更新包，客户端按 `source=null` 处理 |
| **旧版客户端兼容** | 旧版客户端收到带 `source` 字段的更新包，忽略未知字段，走服务端下载 |
| **配置文件向后兼容** | 不包含 `modplatform` 配置的旧版 `config.toml`，pack 跳过模组识别 |
| **Java 版本兼容** | 验证 Java 8/11/17 下客户端功能正常 |
| **网络环境** | 测试 CDN 超时、断网重连下的优雅降级 |
| **跨版本升级** | 从 v1.x 直接升级到支持模组路由的版本，不需要中间版本 |

### 5.5 性能测试

| 指标 | 目标值 | 测试方法 |
|------|--------|---------|
| **pack 速度（无网络）** | 1,000 个 JAR ≤ 10s | 仅计算双重哈希（CF 指纹 + SHA-1 + CRC64+CRC16） |
| **pack 速度（有网络）** | 1,000 个 JAR ≤ 30s | 包括 CF 批量指纹 API 调用 + MR 兜底查询 |
| **指纹匹配准确率** | 知名模组 ≥ 99% | 用 100 个热门 CF 模组测试 |
| **CDN 下载速度** | 相比服务端 ≥ 3x | 对比同一文件从 CDN 和服务端下载耗时 |
| **服务端带宽节省** | 公共模组占比 ≥ 80% | 分析典型整合包模组来源比例 |
| **客户端更新速度** | 混合包加速 ≥ 50% | 对比纯服务端下载 vs 混合路由 |
| **增量 pack 缓存命中** | 第二次 pack 速度提升 ≥ 5x | 第一次全量 pack → 替换 1 个文件 → 第二次 pack |
| **内存占用（增量缓存）** | 10,000 个条目 ≤ 50MB | 模拟大模组包缓存场景 |

---

## 6. 部署与文档

### 6.1 分阶段部署策略

```
Phase 1: 核心引擎 (Week 1-2)
  ├── 范围：
  │   ├── 实现 CF 指纹算法 (MurmurHash2 变体) — fingerprint.rs
  │   ├── CurseForge /fingerprints/fuzzy 批量匹配 — curseforge/fingerprint.rs
  │   ├── Modrinth /version_file/{hash} 单文件查询 — modrinth/hash.rs
  │   ├── ModRouter 识别引擎 — task/mod_router.rs
  │   └── 双重哈希计算工具 (CF指纹 + SHA-1 + CRC64+CRC16)
  ├── 环境：开发环境，mock API 响应
  ├── 可并行任务：
  │   ├── 任务 A: CF 指纹算法实现 + 单元测试
  │   └── 任务 B: Modrinth hash 查询实现 + 单元测试
  ├── 验证：单元测试通过率 ≥ 90%，指纹匹配准确率 ≥ 99%
  └── 验收：pack 阶段能正确识别和路由模组文件

Phase 2: Provider 完整化 (Week 3-4)
  ├── 范围：
  │   ├── 两平台搜索 + 版本列表 + URL 获取（WebUI 用）
  │   ├── 缓存层（内存 + 磁盘）
  │   ├── 限流器集成
  │   ├── REST API 扩展（搜索/详情/版本列表/预览）
  │   ├── 增量 pack 指纹缓存
  │   ├── 离线打包模式
  │   └── 并发安全（文件锁 + 日志脱敏）
  ├── 环境：测试服务器，使用真实 API Key
  ├── 可并行任务：
  │   ├── 任务 A: 缓存层 + 限流器 + 日志脱敏
  │   ├── 任务 B: REST API + WebUI 前端适配
  │   └── 任务 C: 离线模式 + 并发锁 + 增量缓存
  ├── 验证：集成测试通过，WebUI 可搜索和下载模组到 workspace
  └── 验收：管理员可在 WebUI 完成"搜索→下载→放入workspace"闭环

Phase 3: 客户端改造 (Week 5)
  ├── 范围：
  │   ├── UpdateFile 新增 source + downloadUrl 字段（Java）
  │   ├── Work.java CDN 下载分支 + 双源降级
  │   ├── 复用 Servers.multipleAvailableServers() 框架
  │   ├── VersionMeta 序列化兼容性处理
  │   └── 旧版更新包/旧版客户端兼容性
  ├── 环境：搭建完整测试环境（服务端 + 客户端）
  ├── 注意：Java 端的 CRC64+CRC16 校验等不变，仅增加下载来源判断
  ├── 验证：CDN 下载成功回退，CDN 失败自动降级
  └── 验收：完整端到端测试通过（场景 A/B/C/D）

Phase 4: 质量打磨 (Week 6-7)
  ├── 范围：
  │   ├── Week 6: 边界情况处理
  │   │   ├── URl 过期策略（策略 A+C 默认实现）
  │   │   ├── 超大模组包压测（10,000+ JAR）
  │   │   ├── 各种网络异常场景覆盖
  │   │   └── 性能调优
  │   │
  │   └── Week 7: 文档与内测
  │       ├── API 密钥管理指南
  │       ├── 管理员操作手册（含 pack --preview 工作流）
  │       ├── 故障排除指南（见 6.4）
  │       ├── 迁移指南（见 6.5）
  │       ├── mcpatch check-api CLI 命令
  │       └── 内测用户反馈收集并修复关键问题
  ├── 环境：预发布环境
  ├── 验证：性能测试达标，全场景回归通过，CI 构建通过
  └── 验收：文档完整，内测无 Critical 级别问题

Phase 5: 正式发布 (Week 8)
  ├── 范围：
  │   ├── CI/CD 集成（API mock 测试作为 CI 门禁）
  │   ├── 生产环境部署手册
  │   ├── 监控告警配置
  │   └── 稳定版发布
  ├── 环境：生产环境
  ├── 验证：生产环境监控无异常，72h 无 Critical 级别告警
  └── 验收：发布 Release 版本，更新文档
```

### 6.2 交付物清单

| 类别 | 交付物 | 验收标准 |
|------|--------|---------|
| **代码** | Rust `modplatform/` 模块 | 代码审查通过，CI 构建成功 |
| | Rust `task/mod_router.rs` | 单元测试覆盖率 ≥ 80% |
| | Rust `core/fingerprint.rs`（CF 指纹算法） | 与 CurseForge 官方结果一致（验证工具测试通过） |
| | 客户端 `Work.java` 扩展 | 兼容旧版更新包，向下兼容 |
| **配置** | `config.toml` 模组平台配置 | 不包含时不影响现有功能 |
| | 客户端配置零变更 | 无需修改 `mcpatch.yml` |
| | `config.toml` `offline-mode` 选项 | 开启后完全跳过 API 调用 |
| **测试** | 单元测试套件 | 覆盖率 ≥ 70% |
| | 集成测试脚本 | 各场景可重复执行 |
| | 性能测试报告 | pack 速度达标，带宽节省 ≥ 50% |
| **运维** | `mcpatch check-api` CLI | 验证 CF/MR API 连通性 |
| | `mcpatch pack --preview` CLI | 输出路由预览表 |
| | 日志脱敏验证 | 日志中不出现明文 API Key |
| **文档** | API 密钥管理指南 | 步骤清晰，用户可独立完成 |
| | 管理员操作手册 | 覆盖 pack + 搜索 + 路由预览 + 故障排除 |
| | 迁移指南 | 从旧版无痛升级 |

### 6.3 API 密钥管理指南

**CurseForge API Key 获取流程：**

1. 访问 [console.curseforge.com](https://console.curseforge.com/)
2. 注册/登录 CurseForge for Studios 账户
3. 在 API Keys 页面生成新的 API Key
4. **安全配置方式（推荐）**：通过环境变量注入
   - Linux: `export MCPATCH_CF_API_KEY=your_key`
   - Windows: `set MCPATCH_CF_API_KEY=your_key`
   - 程序会优先读取环境变量，其次读取 `config.toml`
5. 备用方式：将 Key 填入 `config.toml` 的 `modplatform.curseforge.api-key` 字段
   - **权限建议**：`chmod 600 config.toml`（仅所有者可读）

**Modrinth Token 获取流程：**

1. 访问 [modrinth.com/settings/api](https://modrinth.com/settings/api)
2. 注册/登录 Modrinth 账户
3. 在 API Tokens 页面创建新 Token（权限选择 "Read projects and versions"）
4. 将 Token 填入 `config.toml` 的 `modplatform.modrinth.api-token` 字段（可选）

> **注意：** API Key/Token **仅配置在服务端**，绝不分发到玩家端。玩家端零配置。
> **安全提醒：** API Key 一旦出现在日志中即为安全事件，验证时请使用 `mcpatch check-api` 命令而非直接打印配置。

### 6.4 故障排除指南

```
常见问题及解决方案：

Q1: pack 时所有模组都被标记为 "server"
  ├── 检查：网络连通性（能否访问 api.curseforge.com / api.modrinth.com）
  ├── 检查：CF API Key 是否有效（运行 mcpatch check-api）
  ├── 检查：是否开启了 offline-mode
  └── 如果以上均正常，模组可能是完全自研的，不受影响

Q2: pack --preview 显示错误识别
  ├── 检查：模组文件是否被修改过（SHA-1 变化会导致匹配失败）
  ├── 检查：模组是否来自非 CurseForge/Modrinth 来源
  └── 确认后执行 pack，私有模组会正确走服务端分发

Q3: 客户端更新时所有文件都从服务端下载（CDN 未生效）
  ├── 原因：可能是旧版客户端（不支持 source 字段）
  ├── 验证：检查更新包元数据是否包含 source 字段
  └── 解决：更新客户端到支持模组路由的版本

Q4: 客户端 CDN 下载频繁失败
  ├── 原因：网络环境限制（如中国大陆访问 CF CDN 受限）
  ├── 不影响更新：双源降级机制会自动切换到服务端
  └── 优化：管理员可考虑不配置平台 API，所有文件走服务端

Q5: 增量 pack 时模组识别结果异常
  ├── 检查：是否替换了模组文件但保持同名
  ├── 缓存：修改时间和文件大小的变化会触发重新识别
  └── 手动清除缓存：mcpatch cache clear curseforge
```

### 6.5 迁移指南

```
从现有部署（无模组路由）升级流程：

  步骤 1: 备份
  ├── 备份 config.toml
  ├── 备份整个 public/ 目录
  └── 备份 index.json

  步骤 2: 更新服务端
  ├── 替换 manager 二进制文件
  ├── 在 config.toml 中添加 [modplatform] 配置段
  └── 配置 CF API Key（环境变量或 config.toml）

  步骤 3: 验证
  ├── 运行 mcpatch check-api 确认 API 连通
  ├── 运行 mcpatch pack --preview 预览路由结果
  └── 确认预览结果中的 source 分配符合预期

  步骤 4: 首次路由包发布
  ├── 执行 pack 命令（使用现有的版本号规则）
  ├── 生成的更新包会包含 source + downloadUrl 字段
  ├── 旧版客户端不受影响（忽略未知字段）
  └── 新版客户端自动走 CDN 下载

  步骤 5: 客户端升级（可选，不强制）
  └── 更新客户端到支持模组路由的版本以享受 CDN 加速

  回滚方案：
  ├── 恢复备份的 config.toml
  ├── 恢复备份的 public/ 目录
  ├── 恢复备份的 index.json
  └── 替换回旧版 manager 二进制文件
```

---

## 7. 风险评估与应对措施

### 7.1 API 变更与版本兼容

| 风险描述 | 概率 | 影响 | 应对措施 |
|----------|------|------|----------|
| CurseForge `/fingerprints/fuzzy` 返回格式变更 | 低 | **极高**（pack 核心路径） | 对关键字段做 Optional 处理；集成测试设为 CI 门禁 |
| Modrinth `/version_file/{hash}` 废弃 | 低 | 中 | 切换到备用端点 `/version/{versionId}/file` |
| CurseForge API Key 策略变更 | 中 | 高 | 关注官方公告，预留多 Key 轮换机制；支持环境变量注入 |
| 下载链接格式/域名变更 | 低 | 中 | 客户端 CDN 下载逻辑与 URL 格式解耦 |
| **CurseForge 指纹算法变更** | **极低** | **极高** | 指纹算法固化在代码中，集成测试验证算出的指纹与服务端返回一致 |

**应对策略：** 建立 API 监控告警机制，每周运行集成测试套件验证 API 兼容性，CI 门禁阻断 API 变更导致的构建失败。

### 7.2 CDN 访问与网络限制

| 风险描述 | 概率 | 影响 | 应对措施 |
|----------|------|------|----------|
| CurseForge/Modrinth CDN 在中国大陆访问受限 | 中 | 中 | **双源降级机制自动兜底**，玩家无需手动配置 |
| CDN 下载速度慢 | 高 | 低 | 自动降级到服务端，速度对比对玩家透明 |
| 服务端 API 访问受限 | 中 | **高**（pack 无法识别） | 支持 HTTP 代理配置到 `config.toml`；offline-mode 手动兜底 |
| 服务端证书验证失败 | 低 | 中 | 提供 `dangerous-accept-invalid-certs` 开关（仅调试用） |
| **pack 阶段 API 超时** | 中 | 中 | 10s 超时后自动降级，已缓存结果不受影响 |

**关于中国大陆玩家的说明：**
由于 CurseForge/Modrinth CDN 在中国大陆可能无法访问，客户端 CDN 下载极大概率会失败。此时**双源降级机制**会自动回退到服务端下载，玩家无感知。管理员也可以选择不配置平台 API，所有模组走服务端分发，与现有行为完全一致。

### 7.3 API 调用频率限制

| 风险描述 | 概率 | 影响 | 应对措施 |
|----------|------|------|----------|
| CurseForge 限流导致 429 | 高 | 高 | 令牌桶限流器 CF ≤ 3 req/s；**指纹批量匹配减少调用次数** |
| Modrinth 限流（300 req/5min） | 中 | 中 | 指纹查询仅在 pack 时进行，频率可控 |
| 大量模组时 API 调用过多 | 中 | 中 | CF 批量指纹单次调用覆盖最多 10,000 个文件 |
| **增量 pack 缓存保护** | — | — | 未变化的文件不重复查询 API |

**限流器配置：**

```rust
pub struct RateLimiterConfig {
    pub max_requests_per_second: u32,      // CF: 3, MR: 5
    pub burst_size: u32,                   // 突发容量
    pub retry_max_attempts: u32,           // 最大重试 3 次
    pub retry_base_delay_ms: u64,          // 初始 1000ms
    pub retry_max_delay_ms: u64,           // 最大 30000ms
    pub retry_jitter: bool,                // 启用随机抖动
}
```

### 7.4 安全风险

| 风险描述 | 概率 | 影响 | 应对措施 |
|----------|------|------|----------|
| API Key 泄露（仅服务端） | 低 | 中 | 支持环境变量注入；日志脱敏；配置文件权限建议 600 |
| **日志中明文打印 API Key** | 中 | **高** | 日志脱敏过滤器（替换为 `****{last4}`）；代码审查重点检查 |
| **错误识别**：私有模组被误判为平台模组 | 低 | **高** | `pack --preview` 预览模式让管理员确认路由结果 |
| 模组 JAR 与平台文件的 CRC64+CRC16 碰撞 | **极低** | 中 | 识别时仅使用 CF 指纹/SHA-1，不依赖 MCPATCH2 内部哈希做识别 |
| 中间人攻击（客户端→CDN） | 低 | 高 | 强制 HTTPS 连接 CDN，客户端校验下载文件的 CRC64+CRC16 哈希 |
| 恶意模组通过 CDN 投毒 | 低 | 高 | CRC64+CRC16 校验确保文件与 pack 时一致 |

### 7.5 数据一致性问题

| 风险描述 | 概率 | 影响 | 应对措施 |
|----------|------|------|----------|
| 平台 CDN 上的文件与 pack 时不一致 | **极低** | 中 | CRC64+CRC16 校验会检测到不一致，触发重新下载 |
| 平台删除旧版本导致 CDN URL 失效 | 中 | 中 | 双源降级到服务端，不影响更新 |
| 重复模组：同一模组在 CF 和 MR 上均有 | 高 | 低 | 优先 CF（更早匹配），pack --preview 中标注"重复" |
| 增量 pack 时识别结果变化 | 中 | 低 | 仅变化的文件重新识别，已有路由结果缓存不变 |
| **回滚后 source 字段残留** | 低 | 低 | 旧版更新包无 source 字段，客户端正确处理 `null` |

---

## 8. 里程碑与时间节点

| 里程碑 | 时间 | 关键交付 | 负责角色 |
|--------|------|---------|---------|
| **M1: 核心引擎完成** | Week 2 | ModRouter + CF 指纹算法 + CF/MR 哈希匹配 + 双重哈希工具 | Rust 后端工程师 |
| **M2: Provider 完整化** | Week 4 | 搜索 + 版本列表 + 缓存层 + REST API + 离线模式 + 并发安全 | Rust 后端工程师 |
| **M3: 客户端改造完成** | Week 5 | 双源下载 + CDN 降级 + 兼容性 + CI 集成测试 | Java 工程师 |
| **M4: 质量打磨** | Week 7 | 边界情况 + 压测 + 文档 + CLI 命令 + 内测反馈 | 全团队 |
| **M5: 正式发布** | Week 8 | Release 版本 + 示例配置 + CI 门禁 | 全团队 |

**人员角色与职责：**

| 角色 | 数量 | 主要职责 |
|------|------|---------|
| Rust 后端工程师 | 1-2 | ModRouter、Provider、缓存、REST API、pack 扩展、CF 指纹算法 |
| Java 工程师 | 1 | 客户端双源下载、CDN 降级、兼容性处理 |
| QA 工程师 | 1 | 测试计划、自动化测试、性能测试 |
| DevOps | 1 | CI/CD、部署、监控、日志脱敏审查 |

**里程碑核心验收标准：**

```
M1: 核心引擎
  ├── CF 指纹算法实现正确（与 CurseForge 官方结果一致）
  ├── pack 命令能正确识别 CurseForge 模组并输出路由结果
  ├── pack 命令能正确识别 Modrinth 模组（CF 未命中时）
  ├── 未识别模组标记为 server 来源
  └── 单元测试覆盖率 ≥ 80%

M3: 客户端改造
  ├── CDN URL 存在时从 CDN 下载，CRC64+CRC16 校验通过
  ├── CDN URL 不存在/失效时自动从服务端下载
  ├── 旧版更新包（无 source 字段）正常走服务端下载
  ├── 旧版客户端（不支持 source 字段）接收新包后正常走服务端下载
  └── 完整端到端测试通过（场景 A/B/C/D）

M4: 质量打磨
  ├── 性能测试达标（pack 1,000 文件 ≤ 30s）
  ├── 文档完整（密钥指南 + 管理员手册 + 故障排除 + 迁移指南）
  ├── CLI 命令可用（mcpatch check-api + pack --preview）
  └── 日志脱敏审查通过

M5: 正式发布
  ├── 性能测试达标（pack 1000 文件 ≤ 30s）
  ├── 文档完整
  ├── 生产环境运行 72h 无 Critical 级别告警
  └── CI 集成测试通过（含 API mock）
```

---

## 9. 附录

### 附录 A：新增/修改文件清单

#### Rust 服务端

```
McPatch2/manager/src/
├── modplatform/                            # [新增] 模组平台模块
│   ├── mod.rs                             # 模块入口，pub mod 声明 + Provider trait
│   ├── types.rs                           # 统一数据模型（PlatformId, UpdateFileEntry, ModSearchResult 等）
│   ├── cache.rs                           # 缓存层（内存 + 磁盘，TTL + RwLock）
│   ├── rate_limiter.rs                    # 令牌桶限流器
│   ├── error.rs                           # 平台错误类型（AuthFailed, NetworkError, RateLimited 等）
│   ├── curseforge/
│   │   ├── mod.rs                         # CurseForge Provider 模块入口
│   │   ├── client.rs                      # API 客户端封装（reqwest, 鉴权, 限流器集成）
│   │   ├── search.rs                      # 模组搜索
│   │   ├── versions.rs                    # 版本列表获取
│   │   ├── download.rs                    # 下载 URL 获取
│   │   └── fingerprint.rs                 # ★ 批量指纹匹配（POST /fingerprints/fuzzy）
│   └── modrinth/
│       ├── mod.rs                         # Modrinth Provider 模块入口
│       ├── client.rs                      # API 客户端封装（reqwest, User-Agent, 可选 Token）
│       ├── search.rs                      # 模组搜索（facets 语法）
│       ├── versions.rs                    # 版本列表获取
│       └── hash.rs                        # ★ SHA-1 哈希查找（GET /version_file/{hash}）
│
├── task/
│   ├── mod.rs                             # [修改] 新增 pub mod mod_router
│   ├── pack.rs                            # [修改] pack 流程中插入模组识别步骤
│   └── mod_router.rs                      # [新增] Pack 阶段模组识别引擎（ModRouter）
│
├── core/
│   ├── mod.rs                             # [修改] 新增 pub mod fingerprint
│   ├── file_hash.rs                       # [修改] 若需要新增 SHA-1 计算函数
│   └── fingerprint.rs                     # [新增] CurseForge 指纹算法（MurmurHash2 64位变体）
│
├── config/
│   ├── mod.rs                             # [修改] 新增 ModPlatformConfig 配置结构体
│   └── core_config.rs                     # [修改] 新增 modplatform 配置段解析
│
├── web/api/
│   ├── mod.rs                             # [修改] 注册 modplatform 路由
│   └── modplatform/                       # [新增] 模组平台 REST API
│       ├── mod.rs                         # 路由注册
│       ├── search.rs                      # POST /api/modplatform/search
│       ├── detail.rs                      # GET /api/modplatform/mod/{platform}/{id}
│       ├── versions.rs                    # GET /api/modplatform/mod/{platform}/{id}/versions
│       ├── download.rs                    # POST /api/modplatform/download
│       └── preview.rs                     # POST /api/modplatform/pack/preview
│
├── utility/
│   └── mod.rs                             # [修改] 新增日志脱敏工具函数

McPatch2/manager/Cargo.toml                # [修改] 新增依赖：reqwest, sha2, serde_json, tokio, murmur2 等
McPatch2/manager/test/config.toml          # [修改] 新增 [modplatform] 测试配置段
```

#### Java 客户端

```
Mcpatch2JavaClient/src/main/java/com/github/balloonupdate/mcpatch/client/
├── data/
│   └── FileChange.java                    # [修改] UpdateFile 内部类新增 source + downloadUrl 字段
│
├── Work.java                              # [修改] 新增 CDN 下载分支 + 双源降级逻辑
│
├── network/
│   ├── Servers.java                       # [修改] 复用 multipleAvailableServers() 框架支持双源
│   └── impl/
│       └── HttpProtocol.java              # [修改] 若需要扩展 CDN 下载相关参数

# 注意：客户端无新增文件，所有变更均为修改现有文件
```

#### WebUI 前端

```
McPatch2/web/src/
├── api/
│   └── modplatform.js                     # [新增] 模组平台 API 封装
│
├── pages/
│   └── Dashboard/
│       └── ModPlatform/                   # [新增] 模组平台管理页面
│           ├── index.jsx                  # 主页面（搜索 + 路由预览入口）
│           ├── SearchPanel.jsx            # 跨平台搜索面板
│           └── RoutePreview.jsx           # pack --preview 结果展示
│
└── router/
    └── index.jsx                          # [修改] 注册 ModPlatform 路由
```

---

### 附录 B：变更摘要（v2.0 → v2.1）

| 类别 | v2.0 问题 | v2.1 修正 |
|------|----------|-----------|
| **数据模型** | `FingerprintRequest.fingerprints` 类型为 `Vec<String>` | 修正为 `Vec<i64>`，CurseForge 使用 64 位数字指纹 |
| | `UpdateFileEntry` 未区分临时字段与持久字段 | 明确 `sha1` / `cf_fingerprint` 仅用于 pack 阶段识别，最终写入元数据只保留 `mcpatch_hash` |
| | 客户端数据模型描述不清晰 | 明确 `UpdateFile` 仅增加 `source` + `downloadUrl` 两字段，无需新增数据类 |
| **架构设计** | 缺少平台无关的 Provider 抽象层 | 新增 `ModPlatformProvider` trait，CF/MR 各自实现 |
| | 未明确增量 pack 的缓存复用策略 | 新增 `fingerprint_cache`（内存）+ 文件路径+修改时间+大小三重校验 |
| | 客户端下载能力描述模糊 | 明确 CDN 下载复用 `HttpProtocol`，双源降级复用 `Servers.multipleAvailableServers()` |
| **配置文件** | 客户端 `mcpatch.yml` 被建议修改 | 修正为"客户端配置文件零变更" |
| | 缺少缓存配置项 | 新增 `[modplatform.cache]` 完整缓存配置段 |
| | 缺少代理配置 | 新增 `[modplatform.proxy]` 可选代理配置 |
| **API 设计** | 缺少 pack 预览端点 | 新增 `POST /api/modplatform/pack/preview` |
| | REST API 路由设计不完整 | 补充了搜索、详情、版本列表、下载、预览的完整路由表 |
| **安全与运维** | 未提及 API Key 脱敏 | 新增日志脱敏策略（替换为 `****{last4}`） |
| | 缺少并发安全设计 | 新增文件锁（`.pack-lock`）+ RwLock 缓存 + 令牌桶限流器 |
| | 缺少离线打包模式 | 新增 `offline-mode` 配置 + 自动超时降级 |
| **测试** | 缺少增量缓存测试用例 | 补充了增量 pack 缓存命中/失效测试 |
| | 缺少并发安全测试用例 | 补充了同一 workspace 并发 pack 测试 |
| | 缺少离线模式测试用例 | 补充了 offline-mode 跳过 API 调用测试 |
| **文档** | 缺少 API 密钥管理指南 | 新增 6.3 节完整指南 |
| | 缺少故障排除指南 | 新增 6.4 节常见问题及解决方案 |
| | 缺少迁移指南 | 新增 6.5 节从旧版升级的完整步骤 |

---

### 附录 C：v2.0 深度审查问题修正对照表

> 以下列出 v2.0 审查中发现的全部 17 项问题及其在 v2.1 中的修正状态。每项均标注审查发现、严重等级、修正方式及对应文档位置。

| # | 审查发现 | 严重等级 | v2.1 修正 | 对应章节 |
|---|---------|---------|-----------|---------|
| 1 | **指纹类型错误**：`FingerprintRequest.fingerprints` 使用 `Vec<String>` 类型，但 CurseForge API 要求 `[i64]` 数字数组。若以字符串发送会导致服务端 422 错误 | **Critical** | 修正为 `Vec<i64>`，代码示例同步更新 | 2.2 节 `FingerprintRequest` 结构体定义 + 3.1 节 `fingerprint.rs` |
| 2 | **哈希体系混淆**：文档中将 CRC64+CRC16（内部哈希）与 CF 指纹（MurmurHash2）、MR 哈希（SHA-1）混用，"哈希校验"语义不明确 | **Critical** | 新增 1.3 节"三套哈希体系并存"对比表，明确各哈希用途、算法、格式、使用方 | 1.3 节 |
| 3 | **缺少统一抽象层**：CF 和 MR 的 Provider 各自独立实现，无统一 Trait，不利于扩展新平台 | High | 新增 `ModPlatformProvider` trait，定义 `search` / `get_mod` / `get_versions` / `get_download_url` / `batch_resolve_fingerprints` / `resolve_hash` 六个接口方法 | 2.2 节 |
| 4 | **增量 pack 缓存设计缺失**：未说明增量 pack 时是否以及如何复用上次的路由结果，导致每次都要重新查询 API | High | 新增 `fingerprint_cache: Arc<Mutex<HashMap<String, FileSource>>>`，缓存 key 为文件路径+修改时间+大小三重校验，TTL 24h | 2.3 节 `ModRouter` + 3.3 节 |
| 5 | **客户端新增数据类不明确**：`ModMeta`、`ModVersionInfo` 等复杂类型出现在客户端章节，容易误导开发者 | High | 明确标注客户端**仅**在 `UpdateFile` 中增加 `source` + `downloadUrl`，所有复杂类型仅服务端使用 | 2.4 节 |
| 6 | **缺少 `pack --preview` 端点**：多个章节提及预览功能但未定义对应 REST API | High | 新增 `POST /api/modplatform/pack/preview` 端点，含完整请求/响应定义 | 3.4 节 |
| 7 | **API Key 日志泄露**：未提及 API Key 在日志中的脱敏处理 | High | 新增日志脱敏规则：输出 `****{last4}`，异常消息自动过滤，错误堆栈移除敏感 headers | 3.8 节 |
| 8 | **并发安全设计缺失**：未说明缓存并发访问的安全性、多个 pack 任务同时执行的处理 | High | 引入 `RwLock`（读多写少场景）、文件锁 `{workspace_dir}/.pack-lock`、令牌桶限流器原子计数器 | 3.8 节 |
| 9 | **离线模式缺失**：未说明管理员在无网络环境下打包时的行为 | Medium | 新增 `offline-mode` 配置项 + 自动超时降级逻辑 + 增量缓存保护机制 | 3.7 节 |
| 10 | **客户端配置文件描述错误**：客户端章节提及可能需要修改 `mcpatch.yml` | Medium | 修正为"客户端配置文件零变更"，CDN 代理通过 JVM 参数解决 | 2.5 节 |
| 11 | **URL 过期策略单一**：仅考虑客户端降级，未说明 pack 时预防性检查 | Medium | 推荐**策略A+策略C**组合：pack 时 HEAD 验证 URL + 客户端双源降级，可选策略B补充 | 3.6 节 |
| 12 | **缓存配置不完整**：仅提及 TTL，缺少缓存最大条目数、磁盘持久化、失效策略 | Medium | 新增 `[modplatform.cache]` 完整配置段：`ttl-searches` / `ttl-mod-detail` / `ttl-versions` / `max-entries` / `ttl-fingerprint-cache` | 2.5 节 + 3.3 节 |
| 13 | **缺少代理配置**：服务端在受限网络环境访问平台 API 的场景未覆盖 | Medium | 新增 `[modplatform.proxy]` 可选 HTTP 代理配置（host + port） | 2.5 节 |
| 14 | **测试用例覆盖不足**：缺少增量缓存、并发安全、离线模式等关键场景的测试用例 | Medium | 补充增量 pack 缓存命中/失效、并发 pack 文件锁、offline-mode、CF 指纹算法正确性等测试项 | 5.1 节 + 5.2 节 |
| 15 | **非 JAR 文件处理策略缺失**：.zip 或其他压缩格式模组的处理方式未定义 | Low | 明确 .zip 仅尝试 Modrinth SHA-1 查询，其他文件不参与模组识别 | 3.5 节 |
| 16 | **版本比较机制描述不完整**：仅提及需要比较但未说明具体比较逻辑 | Low | 补充 `ModPlatformVersionComparator` 设计：CF 解析文件名语义版本，MR 使用 `version_number` 字段 | 2.6 节 |
| 17 | **缺少迁移指南**：现有部署用户升级到新版本的步骤未提供 | Low | 新增完整迁移指南：备份→更新服务端→验证→首次路由包发布→客户端升级（可选）→回滚方案 | 6.5 节 |

---

> **文档结束** — MCPATCH2 v2.1 多平台模组下载支持扩展计划
> 最后更新: 2026-06-02