use reqwest::StatusCode;
use serde::Deserialize;

use crate::modplatform::error::PlatformError;
use crate::modplatform::modrinth::client::ModrinthClient;
use crate::modplatform::types::{ModVersionInfo, PlatformId};

#[derive(Deserialize)]
struct MRVersionsResponse {
    id: String,
    project_id: String,
    name: Option<String>,
    version_number: String,
    game_versions: Vec<String>,
    loaders: Vec<String>,
    files: Vec<MRVersionFile>,
    date_published: String,
    version_type: String,
}

#[derive(Deserialize, Clone)]
struct MRVersionFile {
    url: String,
    filename: String,
    size: u64,
}

impl ModrinthClient {
    pub async fn get_versions(&self, mod_id: &str, game_version: Option<&str>, mod_loader: Option<&str>) -> Result<Vec<ModVersionInfo>, PlatformError> {
        self.limiter.acquire();

        let mut url = format!("{}/project/{}/version", self.base_url, mod_id);
        let mut params: Vec<String> = Vec::new();
        if let Some(gv) = game_version {
            params.push(format!("game_versions={}", gv));
        }
        if let Some(ml) = mod_loader {
            params.push(format!("loaders={}", ml.to_lowercase()));
        }
        if !params.is_empty() {
            url.push_str(&format!("?{}", params.join("&")));
        }

        let resp = self.http_client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await
            .map_err(|e| PlatformError::NetworkError(e.to_string()))?;

        match resp.status() {
            StatusCode::OK => {
                let versions: Vec<MRVersionsResponse> = resp.json()
                    .await
                    .map_err(|e| PlatformError::ParseError(e.to_string()))?;
                Ok(versions.into_iter().map(|v| {
                    let file_info = v.files.first().cloned().unwrap_or(MRVersionFile {
                        url: String::new(),
                        filename: String::new(),
                        size: 0,
                    });
                    ModVersionInfo {
                        platform: PlatformId::Modrinth,
                        version_id: v.id,
                        mod_id: v.project_id,
                        mod_name: v.name.unwrap_or_default(),
                        version_number: v.version_number,
                        game_versions: v.game_versions,
                        mod_loaders: v.loaders,
                        download_url: file_info.url,
                        filename: file_info.filename,
                        file_size: file_info.size,
                        release_date: v.date_published,
                        release_type: v.version_type,
                        sha1_hash: None,
                    }
                }).collect())
            },
            StatusCode::NOT_FOUND => {
                Err(PlatformError::NotFound("模组未找到".to_string()))
            },
            status => {
                Err(PlatformError::NetworkError(format!("HTTP {}", status)))
            },
        }
    }
}
