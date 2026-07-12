use std::{
    collections::HashMap,
    future::Future,
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, Response, StatusCode},
};
use tokio::sync::Mutex;
use tower::{Layer, Service};

/// Max tracked client keys before stale entries are evicted.
const EVICT_THRESHOLD: usize = 1024;

#[derive(Clone)]
pub struct RateLimitLayer {
    limiter: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimitLayer {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            limiter: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimit<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimit {
            inner,
            limiter: self.limiter.clone(),
            max_requests: self.max_requests,
            window: self.window,
        }
    }
}

#[derive(Clone)]
pub struct RateLimit<S> {
    inner: S,
    limiter: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

/// Identify the client for rate-limiting purposes.
///
/// Behind Railway's edge proxy the TCP peer is the proxy, not the client;
/// the proxy appends the real client IP as the LAST entry of
/// `X-Forwarded-For` (earlier entries are client-supplied and spoofable).
/// Falls back to the socket peer address for direct connections.
fn client_key(req: &Request<Body>) -> Option<String> {
    if let Some(xff) = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(ip) = xff.rsplit(',').map(str::trim).find(|s| !s.is_empty()) {
            return Some(ip.to_string());
        }
    }
    req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip().to_string())
}

impl<S> Service<Request<Body>> for RateLimit<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let now = Instant::now();
        let limiter = self.limiter.clone();
        let max_requests = self.max_requests;
        let window = self.window;
        let key = client_key(&req);
        let mut inner = self.inner.clone();

        Box::pin(async move {
            if let Some(key) = key {
                let mut map = limiter.lock().await;
                if map.len() > EVICT_THRESHOLD {
                    map.retain(|_, entries| {
                        entries.retain(|&t| now.duration_since(t) < window);
                        !entries.is_empty()
                    });
                }
                let entries = map.entry(key).or_default();
                entries.retain(|&t| now.duration_since(t) < window);
                if entries.len() >= max_requests {
                    let resp = Response::builder()
                        .status(StatusCode::TOO_MANY_REQUESTS)
                        .body(Body::from("Rate limit exceeded"))
                        .expect("valid response");
                    return Ok(resp);
                }
                entries.push(now);
            }
            inner.call(req).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req_with_xff(xff: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().uri("/api/simulate");
        if let Some(v) = xff {
            builder = builder.header("x-forwarded-for", v);
        }
        builder.body(Body::empty()).unwrap()
    }

    #[test]
    fn client_key_uses_last_forwarded_for_entry() {
        let req = req_with_xff(Some("1.2.3.4, 10.0.0.7, 203.0.113.9"));
        assert_eq!(client_key(&req), Some("203.0.113.9".to_string()));
    }

    #[test]
    fn client_key_falls_back_to_connect_info() {
        let mut req = req_with_xff(None);
        let addr: SocketAddr = "192.0.2.1:5000".parse().unwrap();
        req.extensions_mut().insert(ConnectInfo(addr));
        assert_eq!(client_key(&req), Some("192.0.2.1".to_string()));
    }

    #[test]
    fn client_key_none_without_any_source() {
        let req = req_with_xff(None);
        assert_eq!(client_key(&req), None);
    }
}
