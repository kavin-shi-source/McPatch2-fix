use axum::extract::State;
use axum::response::Response;
use axum::Json;
use base64ct::Encoding;
use serde::Deserialize;
use serde::Serialize;

use crate::web::api::fs::check_path_traversal;
use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    /// 要列目录的路径
    path: String,
}

#[derive(Serialize)]
pub struct ResponseData {
    /// 经过base64编码的完整文件内容
    pub content: String,
}

pub async fn api_download(State(state): State<WebState>, Json(payload): Json<RequestBody>) -> Response {
    // 路径不能为空
    if payload.path.is_empty() {
        return PublicResponseBody::<ResponseData>::err("parameter 'path' is empty, and it is not allowed.");
    }

    let file = state.apppath.working_dir.join(payload.path);

    if let Some(resp) = check_path_traversal(&state.apppath.working_dir, &file) {
        return resp;
    }

    if !file.exists() || !file.is_file() {
        return PublicResponseBody::<ResponseData>::err("file not exists.");
    }

    let data = match tokio::fs::read(&file).await {
        Ok(d) => d,
        Err(_) => return PublicResponseBody::<ResponseData>::err("failed to read file"),
    };

    let b64 = base64ct::Base64::encode_string(&data);

    PublicResponseBody::<ResponseData>::ok(ResponseData { content: b64 })
}
