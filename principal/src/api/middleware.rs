use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Middleware to add a unique request ID to each request
pub async fn request_id(
    mut request: Request,
    next: Next,
) -> Response {
    let request_id = Uuid::new_v4().to_string();
    
    // Add request ID to request headers for logging
    request.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&request_id).unwrap(),
    );
    
    let mut response = next.run(request).await;
    
    // Add request ID to response headers
    response.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&request_id).unwrap(),
    );
    
    response
}

/// Middleware for request logging
pub async fn request_logging(
    request: Request,
    next: Next,
) -> Response {
    let start = std::time::Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();
    
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    tracing::info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        version = ?version,
        "Request started"
    );
    
    let response = next.run(request).await;
    
    let status = response.status();
    let duration = start.elapsed();
    
    tracing::info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        status = %status,
        duration_ms = %duration.as_millis(),
        "Request completed"
    );
    
    response
}

/// Middleware for authentication (placeholder)
pub async fn auth(
    request: Request,
    next: Next,
) -> Response {
    // TODO: Implement proper authentication
    // For now, we'll just pass through all requests
    
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());
    
    if let Some(auth) = auth_header {
        tracing::debug!("Auth header present: {}", auth);
    }
    
    next.run(request).await
}

/// Middleware for rate limiting (placeholder)
pub async fn rate_limit(
    request: Request,
    next: Next,
) -> Response {
    // TODO: Implement proper rate limiting
    // For now, we'll just pass through all requests
    
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    tracing::debug!("Request from IP: {}", client_ip);
    
    next.run(request).await
}