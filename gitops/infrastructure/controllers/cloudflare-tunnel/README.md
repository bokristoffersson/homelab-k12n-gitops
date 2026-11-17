# Cloudflare Tunnel Infrastructure

This directory contains the Cloudflare Tunnel infrastructure components.

## Components

- **cloudflared Deployment**: Cloudflare Tunnel daemon that connects to Cloudflare's edge
- **Secrets**: Tunnel credentials and certificates
- **Cloudflare API Token**: sealed secret left from the retired Access setup jobs; no workload reads it

## Files

- `namespace.yaml` - Cloudflare tunnel namespace
- `deployment.yaml` - Cloudflared deployment configuration
- `cloudflare-tunnel-secret-sealed.yaml` - Tunnel credentials (sealed)
- `cloudflared-cert-secret-sealed.yaml` - Origin certificate (sealed)
- `cloudflare-api-token-secret-sealed.yaml` - Cloudflare API token (sealed, currently unused)
- `kustomization.yaml` - Kustomize configuration

## Secret Management

All secrets are sealed using Sealed Secrets. To create/update:

1. Create the unencrypted secret
2. Seal it with `kubeseal`
3. Commit the sealed secret to git

See individual secret files for creation instructions.

