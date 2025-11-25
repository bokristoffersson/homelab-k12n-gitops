# Heatpump API

REST API backend for monitoring heatpump sensor data. Provides read-only CRUD operations for heatpump telemetry data stored in TimescaleDB.

## Architecture

- **Namespace**: `heatpump-api`
- **Database**: Connects to TimescaleDB in `heatpump-mqtt` namespace
- **Port**: 3000 (HTTP API)
- **Health Check**: `/health` endpoint

## Configuration

### Environment Variables

- `DATABASE_URL`: PostgreSQL connection string (auto-constructed from secrets)
- `DATABASE_USER`: Database user (from `timescaledb-secret`)
- `DATABASE_PASSWORD`: Database password (from `timescaledb-secret`)
- `SERVER_HOST`: Server bind address (default: `0.0.0.0`)
- `SERVER_PORT`: Server port (default: `3000`)
- `RUST_LOG`: Logging level (default: `info`)

### Secrets

- `timescaledb-secret`: Database credentials (SealedSecret)
- `ghcr-secret`: GitHub Container Registry credentials for pulling images (SealedSecret)

## API Endpoints

- `GET /health` - Health check
- `GET /api/v1/heatpump` - List heatpump readings (with filters)
- `GET /api/v1/heatpump/latest` - Get latest reading
- `GET /api/v1/heatpump/:ts` - Get reading by timestamp

## Deployment

The application is deployed via FluxCD GitOps. The image is automatically built and pushed by GitHub Actions:

- Image: `ghcr.io/bokristoffersson/heatpump-api:latest`
- Build workflow: `.github/workflows/heatpump-api-ci-cd.yml`

## Verification

```bash
# Check deployment
kubectl get deployment -n heatpump-api

# Check service
kubectl get service -n heatpump-api

# Port forward to test API
kubectl port-forward service/heatpump-api 3000:3000 -n heatpump-api

# Test health endpoint
curl http://localhost:3000/health

# Test API endpoint
curl http://localhost:3000/api/v1/heatpump?limit=10
```

## Dependencies

- TimescaleDB in `heatpump-mqtt` namespace
- Database credentials from `timescaledb-secret` (shared with heatpump-mqtt)

