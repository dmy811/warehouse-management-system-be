use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use tracing::info;
 
use crate::middlewares::request_id_middleware::RequestId;
pub async fn logging_middleware(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let request_id = req
        .extensions()
        .get::<RequestId>()
        .map(|r| r.0.clone())
        .unwrap_or_else(|| "unknown".to_string());
 
    let start = Instant::now();
    let response = next.run(req).await;
    let duration_ms = start.elapsed().as_millis();
    let status = response.status().as_u16();
 
    info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        status = status,
        duration_ms = duration_ms,
        "Request completed"
    );
 
    response
}