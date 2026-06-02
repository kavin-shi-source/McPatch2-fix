use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};

use crate::modplatform::cache::ModPlatformCache;
use crate::modplatform::rate_limiter::RateLimiter;

/// CurseForge API 客户端
pub struct CurseForgeClient {
    pub http_client: reqwest::Client,
    pub api_key: String,
    pub base_url: String,
    pub cache: Arc<ModPlatformCache>,
    pub limiter: Arc<RateLimiter>,
}

impl CurseForgeClient {
    pub fn new(
        api_key: String,
        cache: Arc<ModPlatformCache>,
        limiter: Arc<RateLimiter>,
        proxy_url: Option<&str>,
    ) -> Self {
        let mut builder = reqwest::Client::builder()
            .user_agent("MCPATCH2/2.1");

        if let Some(proxy) = proxy_url {
            if let Ok(p) = reqwest::Proxy::http(proxy) {
                builder = builder.proxy(p);
            }
            if let Ok(p) = reqwest::Proxy::https(proxy) {
                builder = builder.proxy(p);
            }
        }

        Self {
            http_client: builder.build().unwrap(),
            api_key,
            base_url: "https://api.curseforge.com".to_string(),
            cache,
            limiter,
        }
    }

    pub fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key)).unwrap(),
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("MCPATCH2/2.1"),
        );
        headers
    }
}
