pub mod disk_info;
pub mod list;
pub mod upload;
pub mod download;
pub mod make_directory;
pub mod delete;
pub mod sign_file;
pub mod extract_file;
pub mod r#move;

use std::path::Path;

use axum::body::Body;
use axum::response::Response;

use crate::web::api::PublicResponseBody;

/// 检查路径是否有路径遍历风险
/// 返回 None 表示通过检查，返回 Some(Response) 表示拒绝访问
pub fn check_path_traversal(base: &Path, target: &Path) -> Option<Response> {
    let base_canonical = base.canonicalize().ok()?;
    let target_canonical = target.canonicalize().ok()?;

    if !target_canonical.starts_with(&base_canonical) {
        Some(PublicResponseBody::<()>::err("path traversal detected"))
    } else {
        None
    }
}
