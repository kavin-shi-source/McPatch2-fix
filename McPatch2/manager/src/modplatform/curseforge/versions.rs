use reqwest::StatusCode;
use serde::Deserialize;

use crate::modplatform::curseforge::client::CurseForgeClient;
use crate::modplatform::error::PlatformError;
use crate::modplatform::types::{ModVersionInfo, PlatformId};

#[derive(Deserialize)]
struct CFVersionsResponse {
    data: Vec<CFFile>,
}

#[derive(Deserialize)]
struct CFFile {
    id: i64,
    mod_id: i64,
    file_name: String,
    file_size: u64,
    file_date: String,
    release_type: String,
    download_url: String,
    game_versions: Vec<String>,
    mod_loaders: Vec<String>,
}

impl CurseForgeClient {
    pub async fn get_versions(&self, mod_id: &str, game_version: Option<&str>, _mod_loader: Option<&str>) -> Result<Vec<ModVersionInfo>, PlatformError> {
        self.limiter.acquire();

        let mut url = format!("{}/v1/mods/{}/files?pageSize=50", self.base_url, mod_id);
        if let Some(gv) = game_version {
            url.push_str(&format!("&gameVersion={}", urlencoding(gv)));
        }

        let resp = self.http_client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await
            .map_err(|e| PlatformError::NetworkError(e.to_string()))?;

        match resp.status() {
            StatusCode::OK => {
                let body: CFVersionsResponse = resp.json()
                    .await
                    .map_err(|e| PlatformError::ParseError(e.to_string()))?;
                Ok(body.data.into_iter().map(|f| ModVersionInfo {
                    platform: PlatformId::CurseForge,
                    version_id: f.id.to_string(),
                    mod_id: f.mod_id.to_string(),
                    mod_name: String::new(),
                    version_number: f.file_name.clone(),
                    game_versions: f.game_versions,
                    mod_loaders: f.mod_loaders,
                    download_url: f.download_url,
                    filename: f.file_name,
                    file_size: f.file_size,
                    release_date: f.file_date,
                    release_type: f.release_type,
                    sha1_hash: None,
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

fn urlencoding(s: &str) -> String {
    urlencoding::encode(s).to_string()
}
