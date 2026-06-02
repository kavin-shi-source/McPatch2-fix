use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;

use crate::config::core_config::ModPlatformConfig;
use crate::modplatform::cache::ModPlatformCache;
use crate::modplatform::curseforge::client::CurseForgeClient;
use crate::modplatform::modrinth::client::ModrinthClient;
use crate::modplatform::rate_limiter::RateLimiter;
use crate::modplatform::types::PlatformId;
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
                    log::warn!("CurseForge 搜索失败: {:?}", e);
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
                    log::warn!("Modrinth 搜索失败: {:?}", e);
                }
            }
        }
    }

    PublicResponseBody::ok(results)
}

/// 更新模组平台配置
pub async fn api_modplatform_update_config(
    State(state): State<WebState>,
    Json(_config): Json<ModPlatformConfig>,
) -> axum::response::Response {
    // 配置更新需要重启服务才能生效
    PublicResponseBody::err("配置更新后需要重启服务才能生效")
}
