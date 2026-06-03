use axum::extract::State;
use axum::response::Response;
use serde::Serialize;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Serialize)]
pub struct ResponseData {
    pub dev: String,
    pub used: u64,
    pub total: u64,
}

pub async fn api_disk_info(State(state): State<WebState>) -> Response {
    #[allow(unused_mut)]
    let mut path = match canonical_path_string(&state.apppath.working_dir) {
        Some(path) => path,
        None => return PublicResponseBody::<ResponseData>::err("working directory contains invalid unicode or is unavailable."),
    };

    #[cfg(target_os = "windows")]
    if path.starts_with(r"\\?\") {
        path = path[4..].to_owned();
    }

    let one_peta_bytes: u64 = 1 * 1024 * 1024 * 1024 * 1024 * 1024;
    let mut usages = (one_peta_bytes, one_peta_bytes, "none".to_owned());

    let disks = sysinfo::Disks::new_with_refreshed_list();

    for disk in disks.list() {
        let name = match os_str_to_string(disk.name()) {
            Some(name) => name,
            None => continue,
        };
        let mount = match path_to_string(disk.mount_point()) {
            Some(mount) => mount.replace(r"\\", r"\"),
            None => continue,
        };

        if path.starts_with(&mount) {
            let total = disk.total_space();
            let available = disk.available_space();

            usages = (total - available, total, name);
        }
    }

    PublicResponseBody::<ResponseData>::ok(ResponseData {
        used: usages.0,
        total: usages.1,
        dev: usages.2,
    })
}

fn canonical_path_string(path: &std::path::Path) -> Option<String> {
    let canonical = path.canonicalize().ok()?;
    path_to_string(&canonical)
}

fn path_to_string(path: &std::path::Path) -> Option<String> {
    path.to_str().map(str::to_owned)
}

fn os_str_to_string(value: &std::ffi::OsStr) -> Option<String> {
    value.to_str().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::PathBuf;

    use super::{os_str_to_string, path_to_string};

    #[cfg(windows)]
    fn invalid_os_string() -> OsString {
        use std::os::windows::ffi::OsStringExt;
        OsString::from_wide(&[0x0061, 0xD800, 0x0062])
    }

    #[cfg(unix)]
    fn invalid_os_string() -> OsString {
        use std::os::unix::ffi::OsStringExt;
        OsString::from_vec(vec![0x61, 0xFF, 0x62])
    }

    #[test]
    fn invalid_unicode_path_returns_none() {
        let path = PathBuf::from(invalid_os_string());

        assert!(path_to_string(&path).is_none());
    }

    #[test]
    fn invalid_unicode_os_str_returns_none() {
        let value = invalid_os_string();

        assert!(os_str_to_string(value.as_os_str()).is_none());
    }
}
