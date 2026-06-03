use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use tokio::io::AsyncWriteExt;

use crate::config::core_config::ModPlatformConfig;
use crate::modplatform::cache::ModPlatformCache;
use crate::modplatform::curseforge::client::CurseForgeClient;
use crate::modplatform::modrinth::client::ModrinthClient;
use crate::modplatform::rate_limiter::RateLimiter;
use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

/// 模组平台状态
#[derive(Serialize)]
pub struct ModPlatformStatus {
    pub enabled: bool,
    pub curseforge_configured: bool,
    pub modrinth_configured: bool,
}

/// 搜索请求
#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub platform: Option<String>,
    pub game_version: Option<String>,
    pub mod_loader: Option<String>,
}

/// 搜索响应项
#[derive(Serialize)]
pub struct SearchResultItem {
    pub platform: String,
    pub platform_id: String,
    pub name: String,
    pub description: String,
    pub logo_url: Option<String>,
    pub game_versions: Vec<String>,
    pub mod_loaders: Vec<String>,
    pub downloads: u64,
    pub author: String,
}

/// 识别结果
#[derive(Serialize)]
pub struct IdentifyResult {
    pub file_path: String,
    pub identified: bool,
    pub platform: Option<String>,
    pub mod_name: Option<String>,
    pub download_url: Option<String>,
}

/// 获取模组平台状态
pub async fn api_modplatform_status(
    State(state): State<WebState>,
) -> axum::response::Response {
    let config = &state.config.core.mod_platform;
    PublicResponseBody::ok(ModPlatformStatus {
        enabled: !config.curseforge.api_key.is_empty() || config.modrinth.api_token.is_some(),
        curseforge_configured: !config.curseforge.api_key.is_empty(),
        modrinth_configured: config.modrinth.api_token.is_some(),
    })
}

/// 搜索模组
pub async fn api_modplatform_search(
    State(state): State<WebState>,
    Json(req): Json<SearchRequest>,
) -> axum::response::Response {
    let config = &state.config.core.mod_platform;

    let platform = req.platform.as_deref().unwrap_or("all");

    let mut results = Vec::new();

    if platform == "curseforge" || platform == "all" {
        if !config.curseforge.api_key.is_empty() {
            let cache = Arc::new(ModPlatformCache::new(300, 300, 300, 300, 300, 100));
            let limiter = Arc::new(RateLimiter::new(5, 5));
            let proxy = config.proxy.as_ref().map(|p| format!("{}:{}", p.host, p.port));
            let client = CurseForgeClient::new(
                config.curseforge.api_key.clone(),
                cache,
                limiter,
                proxy.as_deref(),
            );
            match client.search_mods(&req.query, req.game_version.as_deref(), req.mod_loader.as_deref()).await {
                Ok(mods) => {
                    for m in mods {
                        results.push(SearchResultItem {
                            platform: m.platform.to_string(),
                            platform_id: m.platform_id,
                            name: m.name,
                            description: m.summary,
                            logo_url: m.icon_url,
                            game_versions: m.game_versions,
                            mod_loaders: m.mod_loaders,
                            downloads: m.download_count,
                            author: String::new(),
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!("CurseForge 搜索失败: {:?}", e);
                }
            }
        }
    }

    if platform == "modrinth" || platform == "all" {
        if config.modrinth.api_token.is_some() {
            let cache = Arc::new(ModPlatformCache::new(300, 300, 300, 300, 300, 100));
            let limiter = Arc::new(RateLimiter::new(5, 5));
            let proxy = config.proxy.as_ref().map(|p| format!("{}:{}", p.host, p.port));
            let client = ModrinthClient::new(
                config.modrinth.api_token.clone(),
                cache,
                limiter,
                proxy.as_deref(),
            );
            match client.search_mods(&req.query, req.game_version.as_deref(), req.mod_loader.as_deref()).await {
                Ok(mods) => {
                    for m in mods {
                        results.push(SearchResultItem {
                            platform: m.platform.to_string(),
                            platform_id: m.platform_id,
                            name: m.name,
                            description: m.summary,
                            logo_url: m.icon_url,
                            game_versions: m.game_versions,
                            mod_loaders: m.mod_loaders,
                            downloads: m.download_count,
                            author: String::new(),
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!("Modrinth 搜索失败: {:?}", e);
                }
            }
        }
    }

    PublicResponseBody::ok(results)
}

/// 更新模组平台配置
pub async fn api_modplatform_update_config(
    State(_state): State<WebState>,
    Json(_config): Json<ModPlatformConfig>,
) -> axum::response::Response {
    // 配置更新需要重启服务才能生效
    PublicResponseBody::<()>::err("配置更新后需要重启服务才能生效")
}

/// 版本列表请求
#[derive(Deserialize)]
pub struct VersionsRequest {
    pub platform: String,
    pub mod_id: String,
    pub game_version: Option<String>,
    pub mod_loader: Option<String>,
}

/// 获取模组版本列表
pub async fn api_modplatform_versions(
    State(state): State<WebState>,
    Json(req): Json<VersionsRequest>,
) -> axum::response::Response {
    let config = &state.config.core.mod_platform;
    let proxy = config.proxy.as_ref().map(|p| format!("{}:{}", p.host, p.port));

    match req.platform.as_str() {
        "curseforge" => {
            if config.curseforge.api_key.is_empty() {
                return PublicResponseBody::<()>::err("CurseForge 未配置 API Key");
            }
            let cache = Arc::new(ModPlatformCache::new(300, 300, 300, 300, 300, 100));
            let limiter = Arc::new(RateLimiter::new(5, 5));
            let client = CurseForgeClient::new(
                config.curseforge.api_key.clone(),
                cache,
                limiter,
                proxy.as_deref(),
            );
            match client.get_versions(&req.mod_id, req.game_version.as_deref(), req.mod_loader.as_deref()).await {
                Ok(versions) => PublicResponseBody::ok(versions),
                Err(e) => PublicResponseBody::<()>::err(&format!("获取版本列表失败: {}", e)),
            }
        }
        "modrinth" => {
            if config.modrinth.api_token.is_none() {
                return PublicResponseBody::<()>::err("Modrinth 未配置 API Token");
            }
            let cache = Arc::new(ModPlatformCache::new(300, 300, 300, 300, 300, 100));
            let limiter = Arc::new(RateLimiter::new(5, 5));
            let client = ModrinthClient::new(
                config.modrinth.api_token.clone(),
                cache,
                limiter,
                proxy.as_deref(),
            );
            match client.get_versions(&req.mod_id, req.game_version.as_deref(), req.mod_loader.as_deref()).await {
                Ok(versions) => PublicResponseBody::ok(versions),
                Err(e) => PublicResponseBody::<()>::err(&format!("获取版本列表失败: {}", e)),
            }
        }
        _ => PublicResponseBody::<()>::err("不支持的平台"),
    }
}

/// 安装模组请求
#[derive(Deserialize)]
pub struct InstallRequest {
    pub platform: String,
    pub mod_name: String,
    pub download_url: String,
    pub filename: String,
}

/// 安装结果
#[derive(Serialize)]
pub struct InstallResult {
    pub file_path: String,
    pub file_size: u64,
}

/// 下载并安装模组到 workspace
pub async fn api_modplatform_install(
    State(state): State<WebState>,
    Json(req): Json<InstallRequest>,
) -> axum::response::Response {
    let workspace_dir = state.apppath.workspace_dir.clone();
    let file_path = workspace_dir.join(&req.filename);

    // 创建 HTTP 客户端并下载文件
    let client = reqwest::Client::builder()
        .user_agent("MCPATCH2/2.1")
        .build()
        .unwrap();

    let resp = match client.get(&req.download_url).send().await {
        Ok(r) => r,
        Err(e) => return PublicResponseBody::<()>::err(&format!("下载失败: {}", e)),
    };

    if !resp.status().is_success() {
        return PublicResponseBody::<()>::err(&format!("下载失败, HTTP {}", resp.status()));
    }

    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => return PublicResponseBody::<()>::err(&format!("读取响应失败: {}", e)),
    };

    let file_size = bytes.len() as u64;

    // 写入 workspace 目录
    let mut file = match tokio::fs::File::create(&file_path).await {
        Ok(f) => f,
        Err(e) => return PublicResponseBody::<()>::err(&format!("创建文件失败: {}", e)),
    };

    if let Err(e) = file.write_all(&bytes).await {
        return PublicResponseBody::<()>::err(&format!("写入文件失败: {}", e));
    }

    tracing::info!(
        "模组下载完成: {} -> {} ({} bytes)",
        req.mod_name,
        req.filename,
        file_size
    );

    PublicResponseBody::ok(InstallResult {
        file_path: req.filename,
        file_size,
    })
}
