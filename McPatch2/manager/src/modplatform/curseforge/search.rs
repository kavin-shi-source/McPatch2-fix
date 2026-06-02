use reqwest::StatusCode;
use serde::Deserialize;

use crate::modplatform::curseforge::client::CurseForgeClient;
use crate::modplatform::error::PlatformError;
use crate::modplatform::types::{ModSearchResult, PlatformId};

#[derive(Deserialize)]
struct CFSearchResponse {
    data: Vec<CFMod>,
}

#[derive(Deserialize)]
struct CFMod {
    id: i64,
    name: String,
    summary: String,
    logo: Option<CFLogo>,
    download_count: f64,
    latest_files: Vec<CFFileInfo>,
}

#[derive(Deserialize)]
struct CFLogo {
    url: String,
}

#[derive(Deserialize)]
struct CFFileInfo {
    game_versions: Vec<String>,
    mod_loaders: Vec<String>,
}

impl CurseForgeClient {
    pub async fn search_mods(&self, query: &str, game_version: Option<&str>, mod_loader: Option<&str>) -> Result<Vec<ModSearchResult>, PlatformError> {
        self.limiter.acquire();

        let mut url = format!("{}/v1/mods/search?gameId=432&searchFilter={}", self.base_url, urlencoding(query));

        if let Some(gv) = game_version {
            url.push_str(&format!("&gameVersion={}", urlencoding(gv)));
        }
        if let Some(ml) = mod_loader {
            url.push_str(&format!("&modLoaderType={}", mod_loader_type(ml)));
        }

        let resp = self.http_client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await
            .map_err(|e| PlatformError::NetworkError(e.to_string()))?;

        match resp.status() {
            StatusCode::OK => {
                let cf_resp: CFSearchResponse = resp.json()
                    .await
                    .map_err(|e| PlatformError::ParseError(e.to_string()))?;
                Ok(cf_resp.data.into_iter().map(|m| ModSearchResult {
                    platform: PlatformId::CurseForge,
                    platform_id: m.id.to_string(),
                    name: m.name,
                    summary: m.summary,
                    icon_url: m.logo.map(|l| l.url),
                    game_versions: m.latest_files.first().map(|f| f.game_versions.clone()).unwrap_or_default(),
                    mod_loaders: m.latest_files.first().map(|f| f.mod_loaders.iter().map(|ml| ml.to_string()).collect()).unwrap_or_default(),
                    download_count: m.download_count as u64,
                }).collect())
            },
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Err(PlatformError::AuthFailed("API Key 无效或未授权".to_string()))
            },
            StatusCode::TOO_MANY_REQUESTS => {
                Err(PlatformError::RateLimited(std::time::Duration::from_secs(60)))
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

fn mod_loader_type(loader: &str) -> String {
    match loader.to_lowercase().as_str() {
        "forge" => "0",
        "fabric" => "4",
        "neoforge" => "6",
        "quilt" => "5",
        _ => "0",
    }
}
