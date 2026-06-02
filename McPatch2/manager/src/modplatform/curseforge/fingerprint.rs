use serde::{Deserialize, Serialize};

use crate::core::fingerprint::calculate_fingerprint;
use crate::modplatform::curseforge::client::CurseForgeClient;
use crate::modplatform::error::PlatformError;
use crate::modplatform::types::{ModSource, PlatformId, UpdateFileEntry};

/// 指纹匹配请求
#[derive(Serialize)]
struct FingerprintRequest {
    fingerprints: Vec<i64>,
}

/// 指纹匹配响应
#[derive(Deserialize)]
struct FingerprintResponse {
    data: FingerprintData,
}

#[derive(Deserialize)]
struct FingerprintData {
    exact_matches: Vec<ExactMatch>,
    partial_matches: Vec<PartialMatch>,
}

#[derive(Deserialize)]
struct ExactMatch {
    id: i64,
    file: CFFingerprintFile,
}

#[derive(Deserialize)]
struct CFFingerprintFile {
    id: i64,
    mod_id: i64,
    file_name: String,
    mod_name: Option<String>,
    download_url: String,
}

#[derive(Deserialize)]
struct PartialMatch {
    id: i64,
    file: CFFingerprintFile,
}

impl CurseForgeClient {
    /// 批量指纹匹配（POST /v1/fingerprints/fuzzy）
    pub async fn batch_resolve_fingerprints(&self, fingerprints: &[i64]) -> Result<Vec<UpdateFileEntry>, PlatformError> {
        self.limiter.acquire();

        let request_body = FingerprintRequest {
            fingerprints: fingerprints.to_vec(),
        };

        let url = format!("{}/v1/fingerprints/fuzzy", self.base_url);

        let resp = self.http_client
            .post(&url)
            .headers(self.build_headers())
            .json(&request_body)
            .send()
            .await
            .map_err(|e| PlatformError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(PlatformError::NetworkError(format!("fingerprint API 返回 {}", resp.status())));
        }

        let data: FingerprintResponse = resp.json()
            .await
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        let mut result = Vec::new();
        for m in data.data.exact_matches {
            result.push(UpdateFileEntry {
                file_path: m.file.file_name.clone(),
                file_size: 0,
                mcpatch_hash: String::new(),
                sha1_hash: None,
                cf_fingerprint: Some(m.id),
                source: Some(ModSource {
                    platform: PlatformId::CurseForge,
                    mod_id: m.file.mod_id.to_string(),
                    mod_name: m.file.mod_name.unwrap_or_default(),
                    version_id: m.file.id.to_string(),
                    version_number: String::new(),
                    download_url: m.file.download_url,
                    filename: m.file.file_name,
                }),
            });
        }

        Ok(result)
    }
}
