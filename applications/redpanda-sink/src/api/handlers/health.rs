/// Health check endpoint handler
/// Returns "OK" with 200 status code
pub async fn health() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_returns_ok() {
        let result = health().await;
        assert_eq!(result, "OK");
    }

    #[tokio::test]
    async fn test_health_always_succeeds() {
        // Health check should never fail
        for _ in 0..10 {
            let result = health().await;
            assert_eq!(result, "OK");
        }
    }
}
