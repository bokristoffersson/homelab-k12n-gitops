# Cloudflare Tunnel Infrastructure

This directory contains the Cloudflare Tunnel infrastructure components.

## Components

- **cloudflared Deployment**: Cloudflare Tunnel daemon that connects to Cloudflare's edge
- **Secrets**: Tunnel credentials and certificates
- **Cloudflare API Token**: For automated Access setup (see below)

## Cloudflare Access Setup

The `cloudflare-api-token` secret is stored here (infrastructure-level) and is used by application-specific setup jobs.

### Setup Jobs

Application-specific Cloudflare Access setup jobs (e.g., for `heatpump-api`) run in this namespace but are defined in their respective application directories:

- `gitops/apps/base/heatpump-api/setup-cloudflare-access-job.yaml`

The job references the `cloudflare-api-token` secret from this namespace.

## Files

- `namespace.yaml` - Cloudflare tunnel namespace
- `deployment.yaml` - Cloudflared deployment configuration
- `cloudflare-tunnel-secret-sealed.yaml` - Tunnel credentials (sealed)
- `cloudflared-cert-secret-sealed.yaml` - Origin certificate (sealed)
- `cloudflare-api-token-secret-sealed.yaml` - Cloudflare API token for Access setup (sealed)
- `kustomization.yaml` - Kustomize configuration

## Secret Management

All secrets are sealed using Sealed Secrets. To create/update:

1. Create the unencrypted secret
2. Seal it with `kubeseal`
3. Commit the sealed secret to git

See individual secret files for creation instructions.

