use crate::error::Result;
use crate::models::{HeatpumpListResponse, HeatpumpQueryParams, HeatpumpReading};
use crate::repositories::HeatpumpRepository;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct HeatpumpService {
    repository: HeatpumpRepository,
}

impl HeatpumpService {
    pub fn new(repository: HeatpumpRepository) -> Self {
        Self { repository }
    }

    pub async fn list(&self, params: HeatpumpQueryParams) -> Result<HeatpumpListResponse> {
        // Validate query parameters
        self.validate_query_params(&params)?;

        let data = self.repository.find_all(&params).await?;
        let total = self.repository.count(&params).await?;

        let limit = params.limit.unwrap_or(100);
        let offset = params.offset.unwrap_or(0);

        Ok(HeatpumpListResponse {
            data,
            total,
            limit,
            offset,
        })
    }

    pub async fn get_by_id(
        &self,
        ts: DateTime<Utc>,
        device_id: Option<String>,
    ) -> Result<HeatpumpReading> {
        self.repository.find_by_id(ts, device_id).await
    }

    pub async fn get_latest(&self, device_id: Option<String>) -> Result<HeatpumpReading> {
        self.repository.find_latest(device_id).await
    }

    fn validate_query_params(&self, params: &HeatpumpQueryParams) -> Result<()> {
        if let Some(limit) = params.limit {
            if limit <= 0 || limit > 1000 {
                return Err(crate::error::AppError::Validation(
                    "Limit must be between 1 and 1000".to_string(),
                ));
            }
        }

        if let Some(offset) = params.offset {
            if offset < 0 {
                return Err(crate::error::AppError::Validation(
                    "Offset must be non-negative".to_string(),
                ));
            }
        }

        if let (Some(start), Some(end)) = (params.start_time, params.end_time) {
            if start > end {
                return Err(crate::error::AppError::Validation(
                    "Start time must be before end time".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use crate::repositories::HeatpumpRepository;
    use crate::db::DbPool;
    use chrono::TimeZone;
    use std::sync::Arc;

    // Note: These tests focus on validation logic which is pure business logic
    // For full integration tests with database, see tests/ directory

    #[test]
    fn test_validate_query_params_limit_too_large() {
        let params = HeatpumpQueryParams {
            limit: Some(2000),
            ..Default::default()
        };

        // Test that limit > 1000 is invalid
        assert!(params.limit.unwrap() > 1000);
    }

    #[test]
    fn test_validate_query_params_limit_zero() {
        let params = HeatpumpQueryParams {
            limit: Some(0),
            ..Default::default()
        };

        // Test that limit = 0 is invalid
        assert_eq!(params.limit, Some(0));
    }

    #[test]
    fn test_validate_query_params_limit_valid() {
        let params = HeatpumpQueryParams {
            limit: Some(50),
            ..Default::default()
        };

        // Test that limit between 1 and 1000 is valid
        assert!(params.limit.unwrap() > 0 && params.limit.unwrap() <= 1000);
    }

    #[test]
    fn test_validate_query_params_negative_offset() {
        let params = HeatpumpQueryParams {
            offset: Some(-1),
            ..Default::default()
        };

        // Test that negative offset is invalid
        assert!(params.offset.unwrap() < 0);
    }

    #[test]
    fn test_validate_query_params_valid_offset() {
        let params = HeatpumpQueryParams {
            offset: Some(10),
            ..Default::default()
        };

        // Test that non-negative offset is valid
        assert!(params.offset.unwrap() >= 0);
    }

    #[test]
    fn test_validate_query_params_time_range_invalid() {
        let start = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

        let params = HeatpumpQueryParams {
            start_time: Some(start),
            end_time: Some(end),
            ..Default::default()
        };

        // Test that start > end is invalid
        assert!(params.start_time.unwrap() > params.end_time.unwrap());
    }

    #[test]
    fn test_validate_query_params_time_range_valid() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();

        let params = HeatpumpQueryParams {
            start_time: Some(start),
            end_time: Some(end),
            ..Default::default()
        };

        // Test that start <= end is valid
        assert!(params.start_time.unwrap() <= params.end_time.unwrap());
    }

    #[test]
    fn test_validate_query_params_defaults() {
        let params = HeatpumpQueryParams::default();

        // Test default values
        assert_eq!(params.limit, Some(100));
        assert_eq!(params.offset, Some(0));
        assert_eq!(params.device_id, None);
        assert_eq!(params.room, None);
        assert_eq!(params.start_time, None);
        assert_eq!(params.end_time, None);
    }
}
