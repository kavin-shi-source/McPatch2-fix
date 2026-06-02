use std::collections::HashMap;
use std::time::Instant;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct ResponseData {
    token: String,
}

/// 速率限制：每分钟最多 5 次尝试
const MAX_LOGIN_ATTEMPTS: usize = 5;
const RATE_LIMIT_WINDOW: std::time::Duration = std::time::Duration::from_secs(60);

fn check_rate_limit(attempts: &mut HashMap<String, Vec<Instant>>, ip: &str) -> bool {
    let now = Instant::now();
    let entries = attempts.entry(ip.to_string()).or_insert_with(Vec::new);

    // 移除超过时间窗口的记录
    entries.retain(|t| now.duration_since(*t) < RATE_LIMIT_WINDOW);

    if entries.len() >= MAX_LOGIN_ATTEMPTS {
        return false;
    }

    entries.push(now);
    true
}

pub async fn api_login(State(state): State<WebState>, headers: HeaderMap, Json(payload): Json<RequestBody>) -> Response {
    // 获取客户端 IP
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .or_else(|| {
            headers.get("x-real-ip")
                .and_then(|v| v.to_str().ok())
        })
        .unwrap_or("unknown")
        .to_string();

    // 检查速率限制
    {
        let mut attempts = state.login_attempts.lock().await;
        if !check_rate_limit(&mut attempts, &client_ip) {
            tracing::warn!("登录频率过高，IP: {}", client_ip);
            return PublicResponseBody::<ResponseData>::err("登录尝试过于频繁，请稍后再试");
        }
    }

    if payload.username.is_empty() || payload.password.is_empty() {
        return PublicResponseBody::<ResponseData>::err("用户名或密码不能为空");
    }

    let mut auth = state.auth.clone();

    if !auth.test_username(&payload.username).await {
        return PublicResponseBody::<ResponseData>::err("用户名或密码错误");
    }

    if !auth.test_password(&payload.password).await {
        return PublicResponseBody::<ResponseData>::err("用户名或密码错误");
    }

    // 生成新的token
    let new_token = auth.regen_token().await;

    if let Err(_) = auth.save().await {
        return PublicResponseBody::<ResponseData>::err("failed to save auth data");
    }

    PublicResponseBody::<ResponseData>::ok(ResponseData { token: new_token })
}
