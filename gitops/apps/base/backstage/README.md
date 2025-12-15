# Backstage Developer Portal

Backstage deployment for the homelab cluster.

## Overview

This directory contains the GitOps configuration for deploying Spotify's Backstage developer portal to the homelab Kubernetes cluster.

## Architecture

- **Namespace**: `backstage`
- **Database**: Uses existing TimescaleDB instance in `heatpump-mqtt` namespace
- **Image**: `ghcr.io/bokristoffersson/backstage:latest`
- **Exposure**: Via Cloudflare Tunnel at `backstage.k12n.com`
- **Authentication**: GitHub OAuth

## Files

- `namespace.yaml` - Backstage namespace
- `deployment.yaml` - Backstage deployment with resource limits
- `service.yaml` - ClusterIP service on port 7007
- `configmap.yaml` - Backstage app-config.yaml
- `database-init-job.yaml` - Job to create backstage database
- `database-secret-sealed.yaml` - Sealed secret for database credentials (PLACEHOLDER - needs manual creation)
- `github-oauth-secret-sealed.yaml` - Sealed secret for GitHub OAuth (PLACEHOLDER - needs manual creation)
- `ghcr-secret-sealed.yaml` - Sealed secret for GHCR image pull (PLACEHOLDER - needs manual creation)
- `SETUP.md` - Detailed setup instructions for manual steps

## Setup

See `SETUP.md` for detailed instructions on:
1. Creating sealed secrets
2. Setting up GitHub OAuth
3. Creating the database
4. Building and pushing the Docker image

## Configuration

The Backstage configuration is in `configmap.yaml`. To customize:

1. Update `configmap.yaml` for GitOps deployments
2. OR update `app-config.yaml` in the application source and rebuild the image

Key configuration points:
- Database connection via environment variables
- GitHub OAuth authentication
- Catalog locations
- Integrations

## Deployment

Deployed via FluxCD GitOps. Changes to manifests are automatically applied.

Monitor deployment:

```bash
kubectl get pods -n backstage
kubectl logs -f deployment/backstage -n backstage
```

## Access

Once deployed, access Backstage at: https://backstage.k12n.com








