use axum::extract::Query;
use axum::extract::State;
use axum::response::Response;
use serde::Deserialize;

use crate::web::api::fs::check_path_traversal;
use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct QueryString {
    /// 压缩包路径
    pub archive: String,

    /// 在压缩包内的文件路径
    pub path: String,
}

pub async fn api_extract_file(State(state): State<WebState>, Query(query): Query<QueryString>) -> Response {
    let archive_path = state.apppath.working_dir.join(&query.archive);
    let file_path = state.apppath.working_dir.join(&query.path);

    if let Some(resp) = check_path_traversal(&state.apppath.working_dir, &archive_path) {
        return resp;
    }

    if let Some(resp) = check_path_traversal(&state.apppath.working_dir, &file_path) {
        return resp;
    }

    if !archive_path.exists() || !archive_path.is_file() {
        return PublicResponseBody::<()>::err("archive not exists.");
    }

    if !file_path.exists() || !file_path.is_file() {
        return PublicResponseBody::<()>::err("file not exists.");
    }

    PublicResponseBody::<()>::ok_no_data()
}
