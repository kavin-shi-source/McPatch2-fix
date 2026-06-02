use std::time::SystemTime;

use axum::extract::State;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;

use crate::web::api::fs::check_path_traversal;
use crate::web::api::PublicResponseBody;
use crate::web::file_status::SingleFileStatus;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    /// 要列目录的路径
    path: String,
}

#[derive(Serialize)]
pub struct ResponseData {
    pub files: Vec<File>,
}

#[derive(Serialize)]
pub struct File {
    pub name: String,
    pub is_directory: bool,
    pub size: u64,
    pub ctime: u64,
    pub mtime: u64,
    pub state: String,
}

#[axum::debug_handler]
pub async fn api_list(State(state): State<WebState>, Json(payload): Json<RequestBody>) -> Response {
    let mut status = state.status.lock().await;

    let dir = state.apppath.working_dir.join(&payload.path);

    if !dir.exists() || !dir.is_dir() {
        return PublicResponseBody::<ResponseData>::err("directory not exists.");
    }

    if let Some(resp) = check_path_traversal(&state.apppath.working_dir, &dir) {
        return resp;
    }

    let mut files = Vec::<File>::new();

    let mut read_dir = match tokio::fs::read_dir(&dir).await {
        Ok(rd) => rd,
        Err(_) => return PublicResponseBody::<ResponseData>::err("failed to read directory"),
    };

    while let Some(entry) = read_dir.next_entry().await.unwrap_or(None) {
        let is_directory = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
        let metadata = entry.metadata().await.ok();

        let status = match entry.path().strip_prefix(&state.apppath.workspace_dir) {
            Ok(ok) => status.get_file_status(&ok.to_str().unwrap_or("").replace("\\", "/")).await,
            Err(_) => SingleFileStatus::Keep,
        };

        files.push(File {
            name: entry.file_name().to_str().unwrap_or("").to_string(),
            is_directory,
            size: if is_directory { 0 } else { metadata.as_ref().map(|m| m.len()).unwrap_or(0) },
            ctime: metadata.as_ref().and_then(|m| m.created().ok()).map(|e| e.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs()).unwrap_or(0),
            mtime: metadata.as_ref().and_then(|m| m.modified().ok()).map(|e| e.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs()).unwrap_or(0),
            state: match status {
                SingleFileStatus::Keep => "keep".to_owned(),
                SingleFileStatus::Added => "added".to_owned(),
                SingleFileStatus::Modified => "modified".to_owned(),
                SingleFileStatus::Missing => "missing".to_owned(),
                SingleFileStatus::Gone => "gone".to_owned(),
                SingleFileStatus::Come => "come".to_owned(),
            },
        });
    }
    
    PublicResponseBody::<ResponseData>::ok(ResponseData { files })
}
