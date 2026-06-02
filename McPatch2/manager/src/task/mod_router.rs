use std::sync::Arc;

use crate::config::core_config::ModPlatformConfig;
use crate::core::fingerprint::{fingerprint_file, sha1_hash};
use crate::diff::disk_file::DiskFile;
use crate::modplatform::cache::ModPlatformCache;
use crate::modplatform::curseforge::client::CurseForgeClient;
use crate::modplatform::error::PlatformError;
use crate::modplatform::modrinth::client::ModrinthClient;
use crate::modplatform::rate_limiter::RateLimiter;
use crate::modplatform::types::{ModSource, PlatformId};

/// Pack 阶段模组识别引擎
pub struct ModRouter {
    cf_client: Option<CurseForgeClient>,
    mr_client: Option<ModrinthClient>,
    config: ModPlatformConfig,
}

impl ModRouter {
    pub fn new(config: ModPlatformConfig) -> Option<Self> {
        let has_cf = !config.curseforge.api_key.is_empty();
        let has_mr = config.modrinth.api_token.is_some();

        if !has_cf && !has_mr {
            return None;
        }

        let cache = Arc::new(ModPlatformCache::new(
            config.cache.ttl_searches,
            config.cache.ttl_mod_detail,
            config.cache.ttl_versions,
            config.cache.ttl_download,
            config.cache.ttl_fingerprint,
            config.cache.max_entries,
        ));

        let cf_limiter = Arc::new(RateLimiter::new(
            config.curseforge.rate_limit,
            config.curseforge.rate_limit,
        ));

        let mr_limiter = Arc::new(RateLimiter::new(
            config.modrinth.rate_limit,
            config.modrinth.rate_limit,
        ));

        let proxy = config.proxy.as_ref().map(|p| format!("{}:{}", p.host, p.port));
        let proxy_str = proxy.as_deref();

        let cf_client = if has_cf {
            Some(CurseForgeClient::new(
                config.curseforge.api_key.clone(),
                cache.clone(),
                cf_limiter,
                proxy_str,
            ))
        } else {
            None
        };

        let mr_client = if has_mr {
            Some(ModrinthClient::new(
                config.modrinth.api_token.clone(),
                cache.clone(),
                mr_limiter,
                proxy_str,
            ))
        } else {
            None
        };

        Some(Self { cf_client, mr_client, config })
    }

    /// 识别文件来源
    ///
    /// 1. 对 .jar 文件先计算 CF 指纹尝试匹配
    /// 2. 如果匹配失败，再计算 SHA-1 尝试 Modrinth
    /// 3. 对 .zip 文件仅尝试 Modrinth SHA-1
    pub async fn resolve_file_source(&self, file: &DiskFile) -> Result<Option<ModSource>, PlatformError> {
        let disk_path = file.disk_file();
        let ext = disk_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext.to_lowercase() {
            "jar" => self.resolve_jar_file(file).await,
            "zip" => self.resolve_zip_file(file).await,
            _ => Ok(None),
        }
    }

    async fn resolve_jar_file(&self, file: &DiskFile) -> Result<Option<ModSource>, PlatformError> {
        let disk_path = file.disk_file();

        if let Some(cf) = &self.cf_client {
            let fp = fingerprint_file(std::fs::File::open(disk_path).map_err(|e| {
                PlatformError::Internal(format!("无法打开文件: {}", e))
            })?).map_err(|e| PlatformError::Internal(e.to_string()))?;

            let results = cf.batch_resolve_fingerprints(&[fp]).await;
            if let Ok(entries) = results {
                if let Some(entry) = entries.into_iter().next() {
                    if let Some(source) = entry.source {
                        return Ok(Some(source));
                    }
                }
            }
        }

        if let Some(mr) = &self.mr_client {
            let sha1 = sha1_hash(std::fs::File::open(disk_path).map_err(|e| {
                PlatformError::Internal(format!("无法打开文件: {}", e))
            })?).map_err(|e| PlatformError::Internal(e.to_string()))?;

            let result = mr.resolve_hash(&sha1).await;
            if let Ok(info) = result {
                return Ok(Some(ModSource {
                    platform: PlatformId::Modrinth,
                    mod_id: info.mod_id,
                    mod_name: info.mod_name,
                    version_id: info.version_id,
                    version_number: info.version_number,
                    download_url: info.download_url,
                    filename: info.filename,
                }));
            }
        }

        Ok(None)
    }

    async fn resolve_zip_file(&self, file: &DiskFile) -> Result<Option<ModSource>, PlatformError> {
        if let Some(mr) = &self.mr_client {
            let disk_path = file.disk_file();
            let sha1 = sha1_hash(std::fs::File::open(disk_path).map_err(|e| {
                PlatformError::Internal(format!("无法打开文件: {}", e))
            })?).map_err(|e| PlatformError::Internal(e.to_string()))?;

            let result = mr.resolve_hash(&sha1).await;
            if let Ok(info) = result {
                return Ok(Some(ModSource {
                    platform: PlatformId::Modrinth,
                    mod_id: info.mod_id,
                    mod_name: info.mod_name,
                    version_id: info.version_id,
                    version_number: info.version_number,
                    download_url: info.download_url,
                    filename: info.filename,
                }));
            }
        }

        Ok(None)
    }

    pub fn is_enabled(&self) -> bool {
        self.cf_client.is_some() || self.mr_client.is_some()
    }
}
