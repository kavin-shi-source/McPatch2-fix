use serde::{Deserialize, Serialize};
use std::fmt;

/// 平台标识
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

/// 平台模组搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSearchResult {
    pub platform: PlatformId,
    pub platform_id: String,
    pub name: String,
    pub summary: String,
    pub icon_url: Option<String>,
    pub game_versions: Vec<String>,
    pub mod_loaders: Vec<String>,
    pub download_count: u64,
}

/// 模组版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVersionInfo {
    pub platform: PlatformId,
    pub version_id: String,
    pub mod_id: String,
    pub mod_name: String,
    pub version_number: String,
    pub game_versions: Vec<String>,
    pub mod_loaders: Vec<String>,
    pub download_url: String,
    pub filename: String,
    pub file_size: u64,
    pub release_date: String,
    pub release_type: String,
    pub sha1_hash: Option<String>,
}

/// 更新文件条目（模组识别结果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFileEntry {
    pub file_path: String,
    pub file_size: u64,
    pub mcpatch_hash: String,
    pub sha1_hash: Option<String>,
    pub cf_fingerprint: Option<i64>,
    pub source: Option<ModSource>,
}

/// 模组来源信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSource {
    pub platform: PlatformId,
    pub mod_id: String,
    pub mod_name: String,
    pub version_id: String,
    pub version_number: String,
    pub download_url: String,
    pub filename: String,
}

/// 下载 URL 条目（带过期时间）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadUrlEntry {
    pub url: String,
    pub expires_at: Option<i64>,
    pub platform: PlatformId,
    pub mod_name: String,
}
