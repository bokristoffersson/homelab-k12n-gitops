# Scripts

| Script | Purpose |
|---|---|
| `rerun-postgres-migration.sh` | Delete the homelab-settings migration Job so Flux recreates it and applies new migrations (`NAMESPACE` and `JOB_NAME` override the defaults). |
| `check-vulnerabilities.sh` | Run `npm audit` on heatpump-web and `cargo audit` on homelab-settings-api locally. |
| `debug-runner.sh` | Inspect the self-hosted GitHub Actions runner pods. |
