# Scripts

| Script | Purpose |
|---|---|
| `validate-gitops.sh` | Render every path Flux reconciles and validate it with kubeconform. Run by `make lint`, the prek hook and CI. |
| `rerun-postgres-migration.sh` | Delete the homelab-settings migration Job so Flux recreates it and applies new migrations (`NAMESPACE` and `JOB_NAME` override the defaults). |
