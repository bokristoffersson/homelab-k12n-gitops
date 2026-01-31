use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{debug, warn};

use crate::api::handlers::AppState;

pub async fn require_jwt_auth(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let jwt_validator = match &state.jwt_validator {
        Some(validator) => validator,
        None => {
            debug!("JWT validation disabled, allowing request");
            return Ok(next.run(request).await);
        }
    };

    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            warn!("Missing or invalid Authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    match jwt_validator.validate_token(token).await {
        Ok(claims) => {
            debug!("JWT validated for user: {}", claims.sub);
            Ok(next.run(request).await)
        }
        Err(e) => {
            warn!("JWT validation failed: {:?}", e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
