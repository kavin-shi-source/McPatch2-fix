pub mod types;
pub mod error;
pub mod cache;
pub mod rate_limiter;
pub mod curseforge;
pub mod modrinth;

use std::future::Future;
use std::pin::Pin;

use crate::modplatform::error::PlatformError;
use crate::modplatform::types::*;

/// 模组平台 Provider 抽象 Trait
pub trait ModPlatformProvider: Send + Sync {
    /// 搜索模组
    fn search(&self, query: &str, game_version: Option<&str>, mod_loader: Option<&str>) -> Pin<Box<dyn Future<Output = Result<Vec<ModSearchResult>, PlatformError>> + Send>>;

    /// 获取模组详情
    fn get_mod(&self, mod_id: &str) -> Pin<Box<dyn Future<Output = Result<ModSearchResult, PlatformError>> + Send>>;

    /// 获取模组版本列表
    fn get_versions(&self, mod_id: &str, game_version: Option<&str>, mod_loader: Option<&str>) -> Pin<Box<dyn Future<Output = Result<Vec<ModVersionInfo>, PlatformError>> + Send>>;

    /// 获取下载 URL
    fn get_download_url(&self, version_id: &str) -> Pin<Box<dyn Future<Output = Result<DownloadUrlEntry, PlatformError>> + Send>>;

    /// 批量指纹匹配（仅 CurseForge）
    fn batch_resolve_fingerprints(&self, fingerprints: &[i64]) -> Pin<Box<dyn Future<Output = Result<Vec<UpdateFileEntry>, PlatformError>> + Send>>;

    /// 哈希查找（仅 Modrinth）
    fn resolve_hash(&self, sha1_hash: &str) -> Pin<Box<dyn Future<Output = Result<ModVersionInfo, PlatformError>> + Send>>;

    /// 返回平台标识
    fn platform_id(&self) -> PlatformId;
}
