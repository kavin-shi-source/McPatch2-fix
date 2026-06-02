use reqwest::StatusCode;
use serde::Deserialize;

use crate::modplatform::error::PlatformError;
use crate::modplatform::modrinth::client::ModrinthClient;
use crate::modplatform::types::{ModSearchResult, PlatformId};

#[derive(Deserialize)]
struct MRSearchResponse {
    hits: Vec<MRHit>,
}

#[derive(Deserialize)]
struct MRHit {
    project_id: String,
    title: String,
    description: String,
    icon_url: Option<String>,
    versions: Vec<String>,
    categories: Vec<String>,
    downloads: u64,
}

impl ModrinthClient {
    pub async fn search_mods(&self, query: &str, game_version: Option<&str>, mod_loader: Option<&str>) -> Result<Vec<ModSearchResult>, PlatformError> {
        self.limiter.acquire();

        let mut url = format!("{}/search?query={}&limit=20", self.base_url, query);

        if game_version.is_some() || mod_loader.is_some() {
            let mut facets: Vec<String> = Vec::new();
            if let Some(gv) = game_version {
                facets.push(format!("[\"versions:{}\"]", gv));
            }
            if let Some(ml) = mod_loader {
                facets.push(format!("[\"categories:{}\"]", ml.to_lowercase()));
            }
            url.push_str(&format!("&facets=[{}]", facets.join(",")));
        }

        let resp = self.http_client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await
            .map_err(|e| PlatformError::NetworkError(e.to_string()))?;

        match resp.status() {
            StatusCode::OK => {
                let mr_resp: MRSearchResponse = resp.json()
                    .await
                    .map_err(|e| PlatformError::ParseError(e.to_string()))?;
                Ok(mr_resp.hits.into_iter().map(|h| {
                    let mod_loaders: Vec<String> = h.categories.iter()
                        .filter(|c| matches!(c.as_str(), "forge" | "fabric" | "neoforge" | "quilt"))
                        .cloned()
                        .collect();
                    ModSearchResult {
                        platform: PlatformId::Modrinth,
                        platform_id: h.project_id,
                        name: h.title,
                        summary: h.description,
                        icon_url: h.icon_url,
                        game_versions: h.versions,
                        mod_loaders,
                        download_count: h.downloads,
                    }
                }).collect())
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
