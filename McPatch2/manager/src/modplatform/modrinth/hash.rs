use reqwest::StatusCode;
use serde::Deserialize;

use crate::modplatform::error::PlatformError;
use crate::modplatform::modrinth::client::ModrinthClient;
use crate::modplatform::types::{ModVersionInfo, PlatformId};

#[derive(Deserialize)]
struct MRFileResponse {
    id: String,
    project_id: String,
    title: Option<String>,
    version_number: String,
    game_versions: Vec<String>,
    loaders: Vec<String>,
    files: Vec<MRFile>,
    date_published: String,
    version_type: String,
}

#[derive(Deserialize)]
struct MRFile {
    url: String,
    filename: String,
    size: u64,
}

impl ModrinthClient {
    /// 通过 SHA-1 哈希查找文件（GET /version_file/{hash}?algorithm=sha1）
    pub async fn resolve_hash(&self, sha1_hash: &str) -> Result<ModVersionInfo, PlatformError> {
        self.limiter.acquire();

        let url = format!("{}/version_file/{}?algorithm=sha1", self.base_url, sha1_hash);

        let resp = self.http_client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await
            .map_err(|e| PlatformError::NetworkError(e.to_string()))?;

        match resp.status() {
            StatusCode::OK => {
                let file: MRFileResponse = resp.json()
                    .await
                    .map_err(|e| PlatformError::ParseError(e.to_string()))?;

                let file_info = file.files.first().ok_or_else(|| {
                    PlatformError::NotFound("版本无文件".to_string())
                })?;

                Ok(ModVersionInfo {
                    platform: PlatformId::Modrinth,
                    version_id: file.id,
                    mod_id: file.project_id,
                    mod_name: file.title.unwrap_or_default(),
                    version_number: file.version_number,
                    game_versions: file.game_versions,
                    mod_loaders: file.loaders,
                    download_url: file_info.url.clone(),
                    filename: file_info.filename.clone(),
                    file_size: file_info.size,
                    release_date: file.date_published,
                    release_type: file.version_type,
                    sha1_hash: Some(sha1_hash.to_string()),
                })
            },
            StatusCode::NOT_FOUND => {
                Err(PlatformError::NotFound("未找到匹配的文件".to_string()))
            },
            StatusCode::TOO_MANY_REQUESTS => {
                Err(PlatformError::RateLimited(std::time::Duration::from_secs(10)))
            },
            status => {
                Err(PlatformError::NetworkError(format!("HTTP {}", status)))
            },
        }
    }
}
