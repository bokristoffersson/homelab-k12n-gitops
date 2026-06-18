/// Health check endpoint. Returns "OK" with 200.
pub async fn health() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_returns_ok() {
        assert_eq!(health().await, "OK");
    }
}
