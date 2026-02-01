
use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tokio::time::Instant;
use tracing::warn;

use super::*;
pub async fn set_server_time(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let mut res = next.run(req).await;
    let elapsed = format!("sever_time: {}us",start.elapsed().as_micros());
    match HeaderValue::from_str(&elapsed) {
        Ok(v) => res.headers_mut().insert(SERVER_TIME_HEADER,v),
        Err(e) => {
            warn!("Parse elapsed time failed: {} for request {:?}", e, res.headers().get(REQUEST_ID_HEADER));
            None
        }
    };
    res
}