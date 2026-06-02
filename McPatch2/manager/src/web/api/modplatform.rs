use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;

use crate::config::core_config::ModPlatformConfig;
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
            // CurseForge 搜索将在 Phase 7 完整实现
            // 此处返回引导提示
        }
    }

    if platform == "modrinth" || platform == "all" {
        if config.modrinth.api_token.is_some() {
            // Modrinth 搜索将在 Phase 7 完整实现
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
