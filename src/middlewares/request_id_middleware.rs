use axum::{
    body::Body,
    http::{HeaderName, HeaderValue, Request},
    middleware::Next,
    response::Response
};
use std::str::FromStr;
use uuid::Uuid;

use crate::constants::headers;

pub async fn request_id_middleware(
    mut req: Request<Body>,
    next: Next
) -> Response {
    let request_id = req
        .headers()
        .get(headers::REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    req.extensions_mut().insert(RequestId(request_id.clone())); // bisa diakses di tempate lain dengan req.extensions().get::<RequestId>()

    let span = tracing::info_span!("request", request_id = %request_id);
    let _enter = span.enter();

    let mut response = next.run(req).await;

    if let (Ok(name), Ok(value)) = (
        HeaderName::from_str(headers::REQUEST_ID_RESPONSE),
        HeaderValue::from_str(&request_id),
    ) {
        response.headers_mut().insert(name, value);
    }
 
    response
}

#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ini kalau mau tetap dapat span jika ada log di thread lain
// use tracing::Instrument;

// tokio::spawn(
//     async {
//         tracing::info!("di task baru");
//     }
//     .instrument(tracing::Span::current())
// );