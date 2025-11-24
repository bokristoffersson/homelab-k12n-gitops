// Integration tests for the heatpump API
// These tests use a test database with generated data
// Set DATABASE_URL environment variable to run tests
// Example: DATABASE_URL=postgresql://user:pass@localhost/db cargo test --test integration_test
//
// Note: Tests run sequentially to avoid interference

use heatpump_api::{
    repositories::HeatpumpRepository, services::HeatpumpService,
};
use heatpump_api::models::HeatpumpQueryParams;
use test_helpers::*;
use chrono::Utc;

mod test_helpers;

fn get_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://testuser:testpass@localhost:5432/testdb".to_string())
}

#[tokio::test]
async fn test_service_list_with_default_params() {
    let database_url = get_database_url();
    let pool = create_test_pool(&database_url).await.expect("Failed to create test pool");
    
    setup_test_schema(&pool).await.expect("Failed to setup schema");
    cleanup_test_data(&pool).await.expect("Failed to cleanup");
    
    // Insert test data
    insert_test_readings(&pool, 5, None, None).await.expect("Failed to insert test data");

    let repository = HeatpumpRepository::new(pool);
    let service = HeatpumpService::new(repository);

    let params = HeatpumpQueryParams::default();
    let result = service.list(params).await;

    assert!(result.is_ok(), "Service list failed: {:?}", result.err());
    let response = result.unwrap();
    assert!(!response.data.is_empty(), "Response data is empty");
    assert_eq!(response.data.len(), 5, "Expected 5 readings, got {}. Data: {:?}", response.data.len(), response.data.iter().map(|r| (r.ts, r.device_id.clone())).collect::<Vec<_>>());
    assert_eq!(response.total, 5, "Expected total 5, got {}", response.total);
}

#[tokio::test]
async fn test_service_list_with_filters() {
    let database_url = get_database_url();
    let pool = create_test_pool(&database_url).await.expect("Failed to create test pool");
    
    setup_test_schema(&pool).await.expect("Failed to setup schema");
    cleanup_test_data(&pool).await.expect("Failed to cleanup");
    
    // Insert test data with specific device_id
    let test_device_id = "test-device-123";
    insert_test_readings(&pool, 3, Some(test_device_id.to_string()), None)
        .await
        .expect("Failed to insert test data");
    
    // Insert data with different device_id
    insert_test_readings(&pool, 2, Some("other-device".to_string()), None)
        .await
        .expect("Failed to insert test data");

    let repository = HeatpumpRepository::new(pool);
    let service = HeatpumpService::new(repository);

    let params = HeatpumpQueryParams {
        device_id: Some(test_device_id.to_string()),
        limit: Some(10),
        offset: Some(0),
        ..Default::default()
    };

    let result = service.list(params).await;
    assert!(result.is_ok(), "Service list failed: {:?}", result.err());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 3, "Expected 3 readings, got {}. Data: {:?}", response.data.len(), response.data.iter().map(|r| (r.ts, r.device_id.clone())).collect::<Vec<_>>());
    assert_eq!(response.total, 3, "Expected total 3, got {}", response.total);
    
    // Verify all readings have the correct device_id
    for reading in &response.data {
        assert_eq!(reading.device_id, Some(test_device_id.to_string()), "Found reading with wrong device_id: {:?}", reading.device_id);
    }
}

#[tokio::test]
async fn test_service_list_with_room_filter() {
    let database_url = get_database_url();
    let pool = create_test_pool(&database_url).await.expect("Failed to create test pool");
    
    setup_test_schema(&pool).await.expect("Failed to setup schema");
    cleanup_test_data(&pool).await.expect("Failed to cleanup");
    
    let test_room = "living-room";
    insert_test_readings(&pool, 4, None, Some(test_room.to_string()))
        .await
        .expect("Failed to insert test data");
    
    insert_test_readings(&pool, 2, None, Some("bedroom".to_string()))
        .await
        .expect("Failed to insert test data");

    let repository = HeatpumpRepository::new(pool);
    let service = HeatpumpService::new(repository);

    let params = HeatpumpQueryParams {
        room: Some(test_room.to_string()),
        limit: Some(10),
        offset: Some(0),
        ..Default::default()
    };

    let result = service.list(params).await;
    assert!(result.is_ok(), "Service list failed: {:?}", result.err());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 4, "Expected 4 readings, got {}. Data: {:?}", response.data.len(), response.data.iter().map(|r| (r.ts, r.room.clone())).collect::<Vec<_>>());
    assert_eq!(response.total, 4, "Expected total 4, got {}", response.total);
}

#[tokio::test]
async fn test_service_list_with_time_range() {
    let database_url = get_database_url();
    let pool = create_test_pool(&database_url).await.expect("Failed to create test pool");
    
    setup_test_schema(&pool).await.expect("Failed to setup schema");
    cleanup_test_data(&pool).await.expect("Failed to cleanup");
    
    let now = Utc::now();
    let start_time = now - chrono::Duration::hours(2);
    let end_time = now - chrono::Duration::hours(1);
    
    // Insert data within time range
    for i in 0..3 {
        let ts = start_time + chrono::Duration::minutes(20 * i as i64);
        insert_test_reading(&pool, None, None, Some(ts)).await.expect("Failed to insert");
    }
    
    // Insert data outside time range
    insert_test_reading(&pool, None, None, Some(now - chrono::Duration::hours(3)))
        .await
        .expect("Failed to insert");
    insert_test_reading(&pool, None, None, Some(now - chrono::Duration::minutes(30)))
        .await
        .expect("Failed to insert");

    let repository = HeatpumpRepository::new(pool);
    let service = HeatpumpService::new(repository);

    let params = HeatpumpQueryParams {
        start_time: Some(start_time),
        end_time: Some(end_time),
        limit: Some(100),
        offset: Some(0),
        ..Default::default()
    };

    let result = service.list(params).await;
    assert!(result.is_ok(), "Service list failed: {:?}", result.err());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 3, "Expected 3 readings, got {}. Data: {:?}", response.data.len(), response.data.iter().map(|r| (r.ts, r.device_id.clone())).collect::<Vec<_>>());
    assert_eq!(response.total, 3, "Expected total 3, got {}", response.total);
}

#[tokio::test]
async fn test_service_list_with_pagination() {
    let database_url = get_database_url();
    let pool = create_test_pool(&database_url).await.expect("Failed to create test pool");
    
    setup_test_schema(&pool).await.expect("Failed to setup schema");
    cleanup_test_data(&pool).await.expect("Failed to cleanup");
    
    insert_test_readings(&pool, 10, None, None).await.expect("Failed to insert test data");

    let repository = HeatpumpRepository::new(pool);
    let service = HeatpumpService::new(repository);

    // First page
    let params = HeatpumpQueryParams {
        limit: Some(5),
        offset: Some(0),
        ..Default::default()
    };

    let result = service.list(params).await;
    assert!(result.is_ok(), "Service list failed: {:?}", result.err());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 5, "Expected 5 readings on first page, got {}", response.data.len());
    assert_eq!(response.total, 10, "Expected total 10, got {}", response.total);
    assert_eq!(response.limit, 5);
    assert_eq!(response.offset, 0);

    // Second page
    let params = HeatpumpQueryParams {
        limit: Some(5),
        offset: Some(5),
        ..Default::default()
    };

    let result = service.list(params).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 5);
    assert_eq!(response.total, 10);
}

#[tokio::test]
async fn test_service_get_latest() {
    let database_url = get_database_url();
    let pool = create_test_pool(&database_url).await.expect("Failed to create test pool");
    
    setup_test_schema(&pool).await.expect("Failed to setup schema");
    cleanup_test_data(&pool).await.expect("Failed to cleanup");
    
    let test_device_id = "latest-test-device";
    let now = Utc::now();
    
    // Insert readings with different timestamps
    insert_test_reading(&pool, Some(test_device_id.to_string()), None, Some(now - chrono::Duration::hours(2)))
        .await.expect("Failed to insert");
    insert_test_reading(&pool, Some(test_device_id.to_string()), None, Some(now - chrono::Duration::hours(1)))
        .await.expect("Failed to insert");
    insert_test_reading(&pool, Some(test_device_id.to_string()), None, Some(now - chrono::Duration::minutes(30)))
        .await.expect("Failed to insert");

    let repository = HeatpumpRepository::new(pool);
    let service = HeatpumpService::new(repository);

    let result = service.get_latest(Some(test_device_id.to_string())).await;
    assert!(result.is_ok());
    let reading = result.unwrap();
    assert_eq!(reading.device_id, Some(test_device_id.to_string()));
    // The latest should be the most recent timestamp
    assert!(reading.ts >= now - chrono::Duration::minutes(31));
}

#[tokio::test]
async fn test_service_get_by_id() {
    let database_url = get_database_url();
    let pool = create_test_pool(&database_url).await.expect("Failed to create test pool");
    
    setup_test_schema(&pool).await.expect("Failed to setup schema");
    cleanup_test_data(&pool).await.expect("Failed to cleanup");
    
    let test_device_id = "get-by-id-device";
    let test_timestamp = Utc::now() - chrono::Duration::hours(1);
    
    insert_test_reading(&pool, Some(test_device_id.to_string()), None, Some(test_timestamp))
        .await.expect("Failed to insert");

    let repository = HeatpumpRepository::new(pool);
    let service = HeatpumpService::new(repository);

    let result = service.get_by_id(test_timestamp, Some(test_device_id.to_string())).await;
    assert!(result.is_ok());
    let reading = result.unwrap();
    assert_eq!(reading.device_id, Some(test_device_id.to_string()));
}

#[tokio::test]
async fn test_service_validation_limit_too_large() {
    let database_url = get_database_url();
    let pool = create_test_pool(&database_url).await.expect("Failed to create test pool");
    
    setup_test_schema(&pool).await.expect("Failed to setup schema");
    cleanup_test_data(&pool).await.expect("Failed to cleanup");

    let repository = HeatpumpRepository::new(pool);
    let service = HeatpumpService::new(repository);

    let params = HeatpumpQueryParams {
        limit: Some(2000),
        ..Default::default()
    };

    let result = service.list(params).await;
    assert!(result.is_err());
    // Should be a validation error
    if let Err(e) = result {
        assert!(matches!(e, heatpump_api::AppError::Validation(_)));
    }
}

#[tokio::test]
async fn test_service_validation_negative_offset() {
    let database_url = get_database_url();
    let pool = create_test_pool(&database_url).await.expect("Failed to create test pool");
    
    setup_test_schema(&pool).await.expect("Failed to setup schema");
    cleanup_test_data(&pool).await.expect("Failed to cleanup");

    let repository = HeatpumpRepository::new(pool);
    let service = HeatpumpService::new(repository);

    let params = HeatpumpQueryParams {
        offset: Some(-1),
        ..Default::default()
    };

    let result = service.list(params).await;
    assert!(result.is_err());
    // Should be a validation error
    if let Err(e) = result {
        assert!(matches!(e, heatpump_api::AppError::Validation(_)));
    }
}

#[tokio::test]
async fn test_service_validation_invalid_time_range() {
    let database_url = get_database_url();
    let pool = create_test_pool(&database_url).await.expect("Failed to create test pool");
    
    setup_test_schema(&pool).await.expect("Failed to setup schema");
    cleanup_test_data(&pool).await.expect("Failed to cleanup");

    let repository = HeatpumpRepository::new(pool);
    let service = HeatpumpService::new(repository);

    let now = Utc::now();
    let params = HeatpumpQueryParams {
        start_time: Some(now),
        end_time: Some(now - chrono::Duration::hours(1)),
        ..Default::default()
    };

    let result = service.list(params).await;
    assert!(result.is_err());
    // Should be a validation error
    if let Err(e) = result {
        assert!(matches!(e, heatpump_api::AppError::Validation(_)));
    }
}

#[tokio::test]
async fn test_service_get_latest_not_found() {
    let database_url = get_database_url();
    let pool = create_test_pool(&database_url).await.expect("Failed to create test pool");
    
    setup_test_schema(&pool).await.expect("Failed to setup schema");
    cleanup_test_data(&pool).await.expect("Failed to cleanup");

    let repository = HeatpumpRepository::new(pool);
    let service = HeatpumpService::new(repository);

    let result = service.get_latest(Some("non-existent-device".to_string())).await;
    assert!(result.is_err());
    // Should be a not found error
    if let Err(e) = result {
        assert!(matches!(e, heatpump_api::AppError::NotFound(_)));
    }
}
