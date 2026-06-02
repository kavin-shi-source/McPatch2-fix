use std::sync::Arc;

use reqwest::header::USER_AGENT;

use crate::modplatform::cache::ModPlatformCache;
use crate::modplatform::rate_limiter::RateLimiter;

/// Modrinth API 客户端
pub struct ModrinthClient {
    pub http_client: reqwest::Client,
    pub api_token: Option<String>,
    pub base_url: String,
    pub cache: Arc<ModPlatformCache>,
    pub limiter: Arc<RateLimiter>,
}

impl ModrinthClient {
    pub fn new(
        api_token: Option<String>,
        cache: Arc<ModPlatformCache>,
        limiter: Arc<RateLimiter>,
        proxy_url: Option<&str>,
    ) -> Self {
        let mut builder = reqwest::Client::builder()
            .user_agent("MCPATCH2/2.1 (balloonupdate@github)");

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
            api_token,
            base_url: "https://api.modrinth.com/v2".to_string(),
            cache,
            limiter,
        }
    }

    pub fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            USER_AGENT,
            reqwest::header::HeaderValue::from_static("MCPATCH2/2.1 (balloonupdate@github)"),
        );
        if let Some(token) = &self.api_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(token).unwrap(),
            );
        }
        headers
    }
}
