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

#[derive(Clone)]
pub struct RateLimitLayer {
    limiter: Arc<Mutex<HashMap<SocketAddr, Vec<Instant>>>>,
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
    limiter: Arc<Mutex<HashMap<SocketAddr, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
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
        let addr = req.extensions().get::<ConnectInfo<SocketAddr>>().copied();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            if let Some(ConnectInfo(addr)) = addr {
                let mut map = limiter.lock().await;
                let entries = map.entry(addr).or_default();
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
