# Backstage Implementation Status

## ✅ Completed Tasks

### Application Structure
- ✅ Created `applications/backstage/` directory structure
- ✅ Created README.md with setup instructions
- ✅ Created `app-config.example.yaml` template
- ✅ Created multi-stage Dockerfile (supports ARM64)
- ✅ Created `.dockerignore` file

### GitOps Deployment
- ✅ Created namespace manifest
- ✅ Created Deployment manifest (following mqtt-input pattern)
- ✅ Created Service manifest (ClusterIP on port 7007)
- ✅ Created ConfigMap with Backstage configuration
- ✅ Created database init job
- ✅ Created kustomization.yaml for base
- ✅ Created homelab overlay kustomization
- ✅ Added Cloudflare Tunnel ingress rule
- ✅ Added FluxCD Kustomization entry

### Documentation
- ✅ Created SETUP.md with detailed manual steps
- ✅ Created README.md in GitOps directory
- ✅ Created placeholder files for sealed secrets with instructions

## ⏳ Pending Manual Steps

These steps require user action:

1. **Scaffold Backstage Application**
   - Run `npx @backstage/create-app@latest` in applications/ directory
   - See `applications/backstage/README.md`

2. **Create GitHub OAuth App**
   - Create OAuth app at https://github.com/settings/developers
   - Note Client ID and Client Secret
   - See `gitops/apps/base/backstage/SETUP.md`

3. **Create Sealed Secrets** (using kubeseal)
   - Database secret (`database-secret-sealed.yaml`)
   - GitHub OAuth secret (`github-oauth-secret-sealed.yaml`)
   - GHCR secret (`ghcr-secret-sealed.yaml` - can reuse from mqtt-input)
   - See `gitops/apps/base/backstage/SETUP.md` for detailed steps

4. **Build and Push Docker Image**
   - Build: `docker build -t ghcr.io/bokristoffersson/backstage:latest applications/backstage/`
   - Push: `docker push ghcr.io/bokristoffersson/backstage:latest`

5. **Create Database** (optional - init job will create it)
   - Or create manually: `CREATE DATABASE backstage;`

## File Structure Created

```
applications/backstage/
├── README.md
├── app-config.example.yaml
├── Dockerfile
├── .dockerignore
└── IMPLEMENTATION_STATUS.md

gitops/apps/base/backstage/
├── namespace.yaml
├── kustomization.yaml
├── deployment.yaml
├── service.yaml
├── configmap.yaml
├── database-init-job.yaml
├── database-secret-sealed.yaml (PLACEHOLDER)
├── github-oauth-secret-sealed.yaml (PLACEHOLDER)
├── ghcr-secret-sealed.yaml (PLACEHOLDER)
├── README.md
└── SETUP.md

gitops/apps/homelab/backstage/
└── kustomization.yaml
```

## Next Steps

1. Complete the manual steps listed above
2. Commit and push all changes to Git
3. FluxCD will automatically deploy Backstage
4. Access at https://backstage.k12n.com

## Notes

- All manifests follow the existing patterns from `mqtt-input` and `redpanda-sink`
- Resource limits set for Raspberry Pi constraints (512Mi-1Gi memory, 500m-1000m CPU)
- Database uses existing TimescaleDB instance in `heatpump-mqtt` namespace
- Authentication configured for GitHub OAuth
- Exposure via Cloudflare Tunnel at `backstage.k12n.com`








