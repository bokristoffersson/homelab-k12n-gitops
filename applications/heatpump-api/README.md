# Heatpump API

A Rust backend API for monitoring heatpump sensor data. This application provides read-only CRUD operations for heatpump telemetry data stored in TimescaleDB.

## Features

- RESTful API for querying heatpump data
- Filter by device_id, room, and time range
- Pagination support
- Latest reading endpoint
- Comprehensive validation
- Full test coverage for business logic

## Architecture

The application follows a layered architecture:

- **Models**: Domain models and data structures
- **Repositories**: Data access layer (database queries)
- **Services**: Business logic layer (validation, orchestration)
- **Handlers**: HTTP request handlers
- **Routes**: API route definitions

## API Endpoints

### Health Check
```
GET /health
```

### List Heatpump Readings
```
GET /api/v1/heatpump?device_id=<id>&room=<room>&limit=<n>&offset=<n>&start_time=<iso8601>&end_time=<iso8601>
```

Query Parameters:
- `device_id` (optional): Filter by device ID
- `room` (optional): Filter by room
- `limit` (optional, default: 100, max: 1000): Number of results to return
- `offset` (optional, default: 0): Pagination offset
- `start_time` (optional): Start time in RFC3339 format
- `end_time` (optional): End time in RFC3339 format

### Get Latest Reading
```
GET /api/v1/heatpump/latest?device_id=<id>
```

Query Parameters:
- `device_id` (optional): Filter by device ID

### Get Reading by Timestamp
```
GET /api/v1/heatpump/:ts?device_id=<id>
```

Path Parameters:
- `ts`: Timestamp in RFC3339 format

Query Parameters:
- `device_id` (optional): Filter by device ID

## Configuration

The application uses environment variables for configuration:

- `DATABASE_URL`: PostgreSQL connection string (required)
- `DATABASE_MAX_CONNECTIONS`: Maximum database connections (default: 10)
- `SERVER_HOST`: Server host (default: 0.0.0.0)
- `SERVER_PORT`: Server port (default: 3000)

## Database Schema

The application expects a `heatpump` table with the following columns:

- `ts` (TIMESTAMPTZ): Timestamp
- `device_id` (TEXT): Device identifier
- `room` (TEXT): Room identifier
- `outdoor_temp` (DOUBLE PRECISION): Outdoor temperature
- `supplyline_temp` (DOUBLE PRECISION): Supply line temperature
- `returnline_temp` (DOUBLE PRECISION): Return line temperature
- `hotwater_temp` (BIGINT): Hot water temperature
- `brine_out_temp` (BIGINT): Brine out temperature
- `brine_in_temp` (BIGINT): Brine in temperature
- `integral` (BIGINT): Integral value
- `flowlinepump_speed` (BIGINT): Flow line pump speed
- `brinepump_speed` (BIGINT): Brine pump speed
- `runtime_compressor` (BIGINT): Compressor runtime
- `runtime_hotwater` (BIGINT): Hot water runtime
- `runtime_3kw` (BIGINT): 3kW runtime
- `runtime_6kw` (BIGINT): 6kW runtime
- `brinepump_on` (BOOLEAN): Brine pump status
- `compressor_on` (BOOLEAN): Compressor status
- `flowlinepump_on` (BOOLEAN): Flow line pump status
- `hotwater_production` (BOOLEAN): Hot water production status
- `circulation_pump` (BOOLEAN): Circulation pump status
- `aux_heater_3kw_on` (BOOLEAN): Auxiliary heater 3kW status
- `aux_heater_6kw_on` (BOOLEAN): Auxiliary heater 6kW status

## Running the Application

```bash
# Set environment variables
export DATABASE_URL="postgresql://user:password@localhost:5432/timescaledb"

# Run the application
cargo run
```

## Running Tests

```bash
# Run unit tests
cargo test

# Run integration tests (requires database)
DATABASE_URL="postgresql://user:password@localhost:5432/timescaledb" cargo test --test integration_test
```

## Development

The project structure:

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library root
├── config.rs            # Configuration management
├── error.rs             # Error types and handling
├── db.rs                # Database connection
├── routes.rs            # API routes
├── models/              # Domain models
│   ├── mod.rs
│   └── heatpump.rs
├── repositories/        # Data access layer
│   ├── mod.rs
│   └── heatpump.rs
├── services/            # Business logic layer
│   ├── mod.rs
│   └── heatpump.rs
└── handlers/            # HTTP handlers
    ├── mod.rs
    └── heatpump.rs
```

## Future Enhancements

- Write/Update operations for heatpump settings
- Real-time WebSocket support
- Authentication and authorization
- Rate limiting
- Caching layer
- Metrics and observability

