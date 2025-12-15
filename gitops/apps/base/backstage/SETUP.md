# Backstage Setup Guide

This guide covers the manual steps required to complete the Backstage deployment.

## Prerequisites

1. Backstage application scaffolded in `applications/backstage/`
2. Docker image built and pushed to GHCR
3. kubeseal installed and configured

## Manual Steps

### 1. Scaffold Backstage Application

If not already done, scaffold the Backstage app:

```bash
cd applications
npx @backstage/create-app@latest
```

When prompted, name it `backstage` and configure as needed.

### 2. Build and Push Docker Image

Build the Docker image:

```bash
docker build -t ghcr.io/bokristoffersson/backstage:latest applications/backstage/
```

Push to GHCR:

```bash
docker push ghcr.io/bokristoffersson/backstage:latest
```

### 3. Create GitHub OAuth App

1. Go to https://github.com/settings/developers
2. Click "New OAuth App"
3. Configure:
   - **Application name**: Backstage Homelab
   - **Homepage URL**: `https://backstage.k12n.com`
   - **Authorization callback URL**: `https://backstage.k12n.com/api/auth/github/handler/frame`
4. Register the app
5. Note the **Client ID** and generate a **Client Secret**

### 4. Create Sealed Secrets

#### Database Secret

Get existing TimescaleDB credentials:

```bash
kubectl get secret timescaledb-secret -n heatpump-mqtt -o jsonpath='{.data.POSTGRES_USER}' | base64 -d && echo
kubectl get secret timescaledb-secret -n heatpump-mqtt -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d && echo
```

Create temporary secret file:

```bash
cat > /tmp/database-secret.yaml <<EOF
apiVersion: v1
kind: Secret
metadata:
  name: backstage-database-secret
  namespace: backstage
type: Opaque
data:
  POSTGRES_USER: $(echo -n 'YOUR_POSTGRES_USER' | base64)
  POSTGRES_PASSWORD: $(echo -n 'YOUR_POSTGRES_PASSWORD' | base64)
  POSTGRES_DB: $(echo -n 'backstage' | base64)
EOF
```

Seal it:

```bash
kubeseal -f /tmp/database-secret.yaml -o gitops/apps/base/backstage/database-secret-sealed.yaml
```

#### GitHub OAuth Secret

Create temporary secret file:

```bash
cat > /tmp/github-oauth-secret.yaml <<EOF
apiVersion: v1
kind: Secret
metadata:
  name: github-oauth-secret
  namespace: backstage
type: Opaque
data:
  CLIENT_ID: $(echo -n 'YOUR_GITHUB_CLIENT_ID' | base64)
  CLIENT_SECRET: $(echo -n 'YOUR_GITHUB_CLIENT_SECRET' | base64)
EOF
```

Seal it:

```bash
kubeseal -f /tmp/github-oauth-secret.yaml -o gitops/apps/base/backstage/github-oauth-secret-sealed.yaml
```

#### GHCR Secret

You can reuse an existing GHCR secret pattern. Copy from `mqtt-input/ghcr-secret-sealed.yaml` and update the namespace:

```bash
cp gitops/apps/base/mqtt-input/ghcr-secret-sealed.yaml gitops/apps/base/backstage/ghcr-secret-sealed.yaml
```

Then edit the file and change:
- `namespace: mqtt-input` → `namespace: backstage`

OR create a new one following the same pattern as the database secret.

### 5. Create Backstage Database

The database init job will create the database automatically, or you can create it manually:

```bash
kubectl exec -it timescaledb-0 -n heatpump-mqtt -- psql -U postgres -c "CREATE DATABASE backstage;"
```

### 6. Deploy

Commit and push all changes. FluxCD will automatically deploy Backstage.

Monitor the deployment:

```bash
kubectl get pods -n backstage
kubectl logs -f deployment/backstage -n backstage
```

### 7. Verify

Once deployed, access Backstage at: https://backstage.k12n.com

You should be redirected to GitHub for authentication.

## Troubleshooting

### Database Connection Issues

Check if the database was created:

```bash
kubectl exec -it timescaledb-0 -n heatpump-mqtt -- psql -U postgres -l | grep backstage
```

### Image Pull Errors

Verify the GHCR secret:

```bash
kubectl get secret ghcr-secret -n backstage
```

### Authentication Issues

Verify GitHub OAuth credentials are correct in the sealed secret.

### Health Checks

Check the health endpoint:

```bash
kubectl port-forward -n backstage service/backstage 7007:7007
curl http://localhost:7007/healthcheck
```

## Configuration

Update `app-config.yaml` in the ConfigMap or in the application source to customize:

- Catalog locations
- Integrations
- Plugins
- Organization details

After updating, rebuild and push the Docker image.








