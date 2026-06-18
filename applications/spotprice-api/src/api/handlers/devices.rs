use crate::api::middleware::AuthenticatedUser;
use crate::api::models::devices::RegisterDeviceRequest;
use crate::repositories::DeviceTokenRepository;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};

const VALID_ENVIRONMENTS: [&str; 2] = ["sandbox", "production"];

/// Register (or refresh) an APNs device token for the authenticated user.
pub async fn register(
    State((pool, _ctx, _validator)): State<crate::auth::AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<RegisterDeviceRequest>,
) -> Result<StatusCode, StatusCode> {
    if req.token.trim().is_empty() || !VALID_ENVIRONMENTS.contains(&req.environment.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    DeviceTokenRepository::upsert(&pool, &req.token, &req.environment, &user.username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

/// Unregister one of the authenticated user's own device tokens (e.g. on logout).
pub async fn unregister(
    State((pool, _ctx, _validator)): State<crate::auth::AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(token): Path<String>,
) -> Result<StatusCode, StatusCode> {
    // Scoped to the caller's tokens; deleting a token you don't own is a no-op.
    DeviceTokenRepository::delete(&pool, &token, &user.username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}
