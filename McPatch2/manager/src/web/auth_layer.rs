use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

use axum::body::Body;
use axum::http::HeaderMap;
use axum::http::Request;
use axum::http::Response;
use tower_layer::Layer;
use tower_service::Service;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

/// 身份认证middleware
#[derive(Clone)]
pub struct AuthLayer {
    webstate: WebState
}

impl AuthLayer {
    pub fn new(webstate: WebState) -> Self {
        Self { webstate }
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthService<S>;

    fn layer(&self, service: S) -> Self::Service {
        AuthService {
            webstate: self.webstate.clone(),
            service
        }
    }
}

#[derive(Clone)]
pub struct AuthService<S> {
    webstate: WebState,
    service: S,
}

impl<S, Req> Service<Request<Req>> for AuthService<S> where 
    S: Service<Request<Req>, Response = Response<Body>>,
    S::Future: Send + 'static,
    // Req: Send + 'static,
    // Rsp: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), S::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Req>) -> Self::Future {
        let uri = req.uri().to_string();
        tracing::debug!("url = {:?}", uri);

        let webstate = self.webstate.clone();

        // 获取token
        let token_header = token_from_headers(req.headers());

        let fut = self.service.call(req);
        
        Box::pin(async move {
            // 如果token验证失败，就不调用后面的逻辑，直接返回错误
            if let Err(reason) = webstate.auth.validate_token(&token_header).await {
                return Ok(PublicResponseBody::<()>::err_token_expired(reason));
            }
            
            // 请求继续往后走
            fut.await
        })
    }
}

fn token_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("token")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};

    use super::token_from_headers;

    #[test]
    fn invalid_token_header_falls_back_to_empty_string() {
        let mut headers = HeaderMap::new();
        headers.insert("token", HeaderValue::from_bytes(&[0xFF, 0xFE]).unwrap());

        assert_eq!(token_from_headers(&headers), "");
    }
}
