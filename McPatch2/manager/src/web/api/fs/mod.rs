pub mod disk_info;
pub mod list;
pub mod upload;
pub mod download;
pub mod make_directory;
pub mod delete;
pub mod sign_file;
pub mod extract_file;
pub mod r#move;

use std::path::Component;
use std::path::Path;

use axum::response::Response;

use crate::web::api::PublicResponseBody;

/// 检查路径是否有路径遍历风险
/// 返回 None 表示通过检查，返回 Some(Response) 表示拒绝访问
pub fn check_path_traversal(base: &Path, target: &Path) -> Option<Response> {
    let base_canonical = base.canonicalize().ok()?;
    let relative_path = match target.strip_prefix(base) {
        Ok(path) => path,
        Err(_) => return Some(PublicResponseBody::<()>::err("path traversal detected")),
    };

    let has_forbidden_component = relative_path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    });

    if has_forbidden_component {
        return Some(PublicResponseBody::<()>::err("path traversal detected"));
    }

    let normalized_target = base_canonical.join(relative_path);
    if !normalized_target.starts_with(&base_canonical) {
        Some(PublicResponseBody::<()>::err("path traversal detected"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::check_path_traversal;

    fn test_dir(label: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mcpatch-fs-test-{label}-{unique}"));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn allows_existing_child_path() {
        let base = test_dir("base-ok");
        let target = base.join("child.txt");
        std::fs::write(&target, "ok").unwrap();

        assert!(check_path_traversal(&base, &target).is_none());
    }

    #[test]
    fn rejects_nonexistent_path_that_escapes_base_dir() {
        let root = test_dir("root");
        let base = root.join("base");
        std::fs::create_dir_all(&base).unwrap();
        let target = base.join("..").join("escape.txt");

        assert!(check_path_traversal(&base, &target).is_some());
    }
}
