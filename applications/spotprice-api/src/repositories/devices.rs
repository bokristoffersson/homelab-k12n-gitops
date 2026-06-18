use crate::db::DbPool;
use crate::error::AppError;
use sqlx::FromRow;

/// A registered APNs device token.
#[derive(Debug, Clone, FromRow)]
pub struct DeviceToken {
    pub token: String,
    pub environment: String,
}

pub struct DeviceTokenRepository;

impl DeviceTokenRepository {
    /// Register (or refresh) a device token. Idempotent on the token.
    pub async fn upsert(
        pool: &DbPool,
        token: &str,
        environment: &str,
        user_sub: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO apns_device_tokens (token, environment, user_sub, last_seen_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (token) DO UPDATE SET
                environment = EXCLUDED.environment,
                user_sub = EXCLUDED.user_sub,
                updated_at = NOW(),
                last_seen_at = NOW()
            "#,
        )
        .bind(token)
        .bind(environment)
        .bind(user_sub)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Remove a device token owned by `user_sub` (e.g. on logout). Scoping the
    /// delete to the owner prevents one authenticated user from deleting another
    /// user's token by its value.
    pub async fn delete(pool: &DbPool, token: &str, user_sub: &str) -> Result<u64, AppError> {
        let result =
            sqlx::query("DELETE FROM apns_device_tokens WHERE token = $1 AND user_sub = $2")
                .bind(token)
                .bind(user_sub)
                .execute(pool)
                .await?;
        Ok(result.rows_affected())
    }

    /// All registered device tokens (used by the notifier).
    pub async fn all(pool: &DbPool) -> Result<Vec<DeviceToken>, AppError> {
        let rows =
            sqlx::query_as::<_, DeviceToken>("SELECT token, environment FROM apns_device_tokens")
                .fetch_all(pool)
                .await?;
        Ok(rows)
    }
}
