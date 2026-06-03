use crate::modplatform::curseforge::client::CurseForgeClient;
use crate::modplatform::error::PlatformError;
use crate::modplatform::types::{DownloadUrlEntry, PlatformId};

impl CurseForgeClient {
    pub async fn get_download_url(&self, mod_id: &str, version_id: &str) -> Result<DownloadUrlEntry, PlatformError> {
        self.limiter.acquire();

        let url = format!("{}/v1/mods/{}/files/{}", self.base_url, mod_id, version_id);

        let resp = self.http_client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await
            .map_err(|e| PlatformError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(PlatformError::NetworkError(format!("HTTP {}", resp.status())));
        }

        #[derive(serde::Deserialize)]
        struct CFFileResponse {
            data: CFFileData,
        }

        #[derive(serde::Deserialize)]
        struct CFFileData {
            download_url: String,
            file_name: String,
            _mod_id: i64,
        }

        let body: CFFileResponse = resp.json()
            .await
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        Ok(DownloadUrlEntry {
            url: body.data.download_url,
            expires_at: None,
            platform: PlatformId::CurseForge,
            mod_name: body.data.file_name,
        })
    }
}
