use std::time::Duration;
use std::time::SystemTime;

use axum::extract::State;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    /// 要下载的文件路径
    path: String,
}

#[derive(Serialize)]
pub struct ResponseData {
    /// 文件的签名数据
    signature: String,
}

pub async fn api_sign_file(State(state): State<WebState>, Json(payload): Json<RequestBody>) -> Response {
    // 路径不能为空
    if payload.path.is_empty() {
        return PublicResponseBody::<ResponseData>::err("parameter 'path' is empty, and it is not allowed.");
    }

    let path = state.apppath.working_dir.join(&payload.path);

    if let Some(resp) = crate::web::api::fs::check_path_traversal(&state.apppath.working_dir, &path) {
        return resp;
    }

    if !path.exists() || !path.is_file() {
        return PublicResponseBody::<ResponseData>::err("file not exists.");
    }

    let username = state.auth.username().await;
    let password = state.auth.password().await;

    let relative_path = match relative_path_string(&state.apppath.working_dir, &path) {
        Some(path) => path,
        None => return PublicResponseBody::<ResponseData>::err("file path contains invalid unicode."),
    };
    let expire = SystemTime::now() + Duration::from_secs(2 * 60 * 60);
    let unix_ts = expire.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

    let core_data = format!("{}:{}", relative_path, unix_ts);
    let full_data = format!("{}:{}@{}", core_data, username, password);
    let digest = hash(&full_data);
    let signature = format!("{}:{}", core_data, digest);

    PublicResponseBody::<ResponseData>::ok(ResponseData { signature })
}

fn hash(text: &impl AsRef<str>) -> String {
    let hash = Sha256::digest(text.as_ref());
    
    base16ct::lower::encode_string(&hash)
}

fn relative_path_string(base: &std::path::Path, path: &std::path::Path) -> Option<String> {
    path.strip_prefix(base).ok()?.to_str().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use super::relative_path_string;

    #[cfg(windows)]
    fn invalid_path() -> PathBuf {
        use std::os::windows::ffi::OsStringExt;
        let os = OsString::from_wide(&[0x0061, 0xD800, 0x0062]);
        PathBuf::from(os)
    }

    #[cfg(unix)]
    fn invalid_path() -> PathBuf {
        use std::os::unix::ffi::OsStringExt;
        PathBuf::from(OsString::from_vec(vec![0x61, 0xFF, 0x62]))
    }

    #[test]
    fn invalid_unicode_relative_path_returns_none() {
        let base = Path::new("root");
        let path = base.join(invalid_path());

        assert!(relative_path_string(base, &path).is_none());
    }
}
