use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use crate::auth::jwt::validate_token;
use crate::config::Config;

pub async fn require_auth(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract config from extensions (set by router layer)
    let config = request.extensions()
        .get::<Config>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let auth = config.auth.as_ref().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let auth_header = request.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());
    
    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    
    validate_token(token, &auth.jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    Ok(next.run(request).await)
}


