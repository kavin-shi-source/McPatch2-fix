use axum::extract::State;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::web::api::fs::check_path_traversal;
use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    /// 要创建的目录的路径
    path: String,
}

pub async fn api_make_directory(State(state): State<WebState>, Json(payload): Json<RequestBody>) -> Response {
    let path = payload.path;

    // 路径不能为空
    if path.is_empty() || path == "/" {
        return PublicResponseBody::<()>::err("parameter 'path' is empty");
    }

    let file = state.apppath.working_dir.join(&path);

    if let Some(resp) = check_path_traversal(&state.apppath.working_dir, &file) {
        return resp;
    }

    if file.exists() {
        return PublicResponseBody::<()>::err("path already exists.");
    }

    match std::fs::create_dir_all(&file) {
        Ok(_) => (),
        Err(err) => return PublicResponseBody::<()>::err(&format!("{:?}", err)),
    }

    PublicResponseBody::<()>::ok_no_data()
}
