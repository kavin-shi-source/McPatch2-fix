use axum::extract::State;
use axum::response::Response;
use axum::Json;
use base64ct::Base64;
use base64ct::Encoding;
use serde::Deserialize;
use serde::Serialize;
use tokio::io::AsyncWriteExt;

use crate::web::api::fs::check_path_traversal;
use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    /// 要上传文件的路径
    path: String,

    /// base64编码的文件内容
    content: String,
}

#[derive(Serialize)]
pub struct ResponseData {
    pub size: u64,
}

pub async fn api_upload_fs(State(state): State<WebState>, Json(payload): Json<RequestBody>) -> Response {
    let path = payload.path;

    // 路径不能为空
    if path.is_empty() {
        return PublicResponseBody::<ResponseData>::err("parameter 'path' is empty");
    }

    let file = state.apppath.working_dir.join(&path);

    if let Some(resp) = check_path_traversal(&state.apppath.working_dir, &file) {
        return resp;
    }

    // 确保父目录存在
    let parent = match file.parent() {
        Some(p) => p,
        None => return PublicResponseBody::<ResponseData>::err("invalid path"),
    };

    if !parent.exists() {
        match std::fs::create_dir_all(parent) {
            Ok(_) => (),
            Err(err) => return PublicResponseBody::<ResponseData>::err(&format!("{:?}", err)),
        }
    }

    let data = match base64ct::Base64::decode_vec(&payload.content) {
        Ok(d) => d,
        Err(_) => return PublicResponseBody::<ResponseData>::err("base64 decode failed"),
    };

    let mut opened = match tokio::fs::File::create(&file).await {
        Ok(f) => f,
        Err(err) => return PublicResponseBody::<ResponseData>::err(&format!("{:?}", err)),
    };

    if opened.write_all(&data).await.is_err() {
        return PublicResponseBody::<ResponseData>::err("failed to write file");
    }

    if opened.flush().await.is_err() {
        return PublicResponseBody::<ResponseData>::err("failed to flush file");
    }

    let metadata = match opened.metadata().await {
        Ok(m) => m,
        Err(_) => return PublicResponseBody::<ResponseData>::err("failed to get metadata"),
    };

    // 清除文件状态缓存
    let mut status = state.status.lock().await;
    status.invalidate();

    PublicResponseBody::<ResponseData>::ok(ResponseData { size: metadata.len() })
}
