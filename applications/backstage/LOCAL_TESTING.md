# Local Testing Guide for Backstage with Kafka Plugin

This guide shows you how to test the Kafka plugin locally by running Backstage on your machine and connecting to the Redpanda cluster in Kubernetes via port-forwarding.

## Prerequisites

- Kubernetes cluster with Redpanda running
- Node.js 22+ installed
- Yarn installed
- kubectl configured to access your cluster

## Steps

### 1. Port-forward to Redpanda

In a separate terminal, set up port-forwarding to the Redpanda service:

```bash
kubectl port-forward -n redpanda-v2 svc/redpanda-v2 9092:9092
```

Keep this terminal running. The port-forward will forward `localhost:9092` to the Redpanda service in your cluster.

### 2. Install Dependencies (if not already done)

```bash
cd applications/backstage
yarn install
```

### 3. Start Backstage Locally

```bash
# From the applications/backstage directory
yarn start
```

This will:
- Start the backend on `http://localhost:7007`
- Start the frontend on `http://localhost:3000`
- Use `app-config.yaml` and `app-config.local.yaml` for configuration

### 4. Access Backstage

Open your browser and navigate to:
- Frontend: http://localhost:3000
- Backend API: http://localhost:7007

### 5. Test the Kafka Plugin

1. Navigate to `/kafka` in the frontend to see the standalone Kafka page
2. Or navigate to any entity page and look for the "Kafka" tab (if the entity has the `kafka.apache.org/consumer-groups` annotation)

## Configuration

The local configuration (`app-config.local.yaml`) overrides the main config to use `localhost:9092` for Kafka, which will be forwarded to your Redpanda cluster.

## Troubleshooting

### Port-forward fails
- Make sure the Redpanda service is running: `kubectl get svc -n redpanda-v2`
- Check if port 9092 is already in use: `lsof -i :9092`

### Kafka plugin shows errors
- Verify the port-forward is active: `curl localhost:9092` (should fail, but confirms port is listening)
- Check backend logs for connection errors
- Verify Redpanda is accessible from the cluster

### Frontend shows white screen
- Check browser console for JavaScript errors
- Verify the plugin was built: `yarn workspace app build`
- Clear browser cache and hard refresh

## Stopping

1. Stop Backstage: Press `Ctrl+C` in the terminal running `yarn start`
2. Stop port-forward: Press `Ctrl+C` in the terminal running `kubectl port-forward`

