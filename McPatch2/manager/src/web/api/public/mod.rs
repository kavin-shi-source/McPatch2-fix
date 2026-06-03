use axum::body::Body;
use axum::extract::Path;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use reqwest::StatusCode;
use tokio::io::AsyncSeekExt;

use crate::utility::filename_ext::GetFileNamePart;
use crate::utility::partial_read::PartialAsyncRead;
use crate::web::api::fs::check_path_traversal;
use crate::web::webstate::WebState;

pub async fn api_public(State(state): State<WebState>, headers: HeaderMap, Path(path): Path<String>) -> Response {
    tracing::info!("+public: {}", path);

    let path = state.apppath.public_dir.join(path);

    if check_path_traversal(&state.apppath.public_dir, &path).is_some() {
        return Response::builder().status(404).body(Body::empty()).unwrap();
    }

    if !path.is_file() {
        return Response::builder().status(404).body(Body::empty()).unwrap();
    }

    let range = parse_range_header(&headers);

    // 检查range参数
    if let Some(range) = &range {
        if range.end < range.start {
            return Response::builder().status(403).body(Body::from("incorrect range")).unwrap();
        }
    }

    let metadata = match tokio::fs::metadata(&path).await {
        Ok(m) => m,
        Err(_) => return Response::builder().status(404).body(Body::empty()).unwrap(),
    };

    let mut file = match tokio::fs::File::options()
        .read(true)
        .open(&path)
        .await
    {
        Ok(f) => f,
        Err(_) => return Response::builder().status(500).body(Body::empty()).unwrap(),
    };

    if let Some(range) = &range {
        if file.seek(std::io::SeekFrom::Start(range.start)).await.is_err() {
            return Response::builder().status(500).body(Body::empty()).unwrap();
        }
    }

    let len = match &range {
        Some(range) => range.end - range.start,
        None => metadata.len(),
    };

    let file = tokio_util::io::ReaderStream::new(PartialAsyncRead::new(file, len));

    let mut builder = Response::builder();

    builder = builder.header(axum::http::header::CONTENT_TYPE, "application/octet-stream");
    builder = builder.header(axum::http::header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", path.filename()));
    builder = builder.header(axum::http::header::CONTENT_LENGTH, format!("{}", len));

    if let Some(range) = &range {
        builder = builder.header(axum::http::header::CONTENT_RANGE, format!("{}-{}/{}", range.start, range.end - 1, metadata.len()));
        builder = builder.status(StatusCode::PARTIAL_CONTENT);
    }

    builder.body(Body::from_stream(file)).unwrap()
}

fn parse_range_header(headers: &HeaderMap) -> Option<std::ops::Range<u64>> {
    headers.get("range")
        .and_then(|e| e.to_str().ok())
        .filter(|e| e.starts_with("bytes="))
        .map(|e| e["bytes=".len()..].split("-"))
        .and_then(|mut e| Some((e.next()?, e.next()?)))
        .and_then(|e| Some((u64::from_str_radix(e.0, 10).ok()?, u64::from_str_radix(e.1, 10).ok()? + 1)))
        .filter(|e| e != &(0, 0))
        .map(|e| e.0..e.1)
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};

    use super::parse_range_header;

    #[test]
    fn invalid_range_header_does_not_panic_and_returns_none() {
        let mut headers = HeaderMap::new();
        headers.insert("range", HeaderValue::from_bytes(&[0xFF, 0xFE]).unwrap());

        assert!(parse_range_header(&headers).is_none());
    }
}
