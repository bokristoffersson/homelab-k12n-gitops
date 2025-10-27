# TimescaleDB Datasource Setup

## Overview

The TimescaleDB datasource is configured using Grafana's **sidecar for datasources** approach as documented in the [Grafana Helm Chart README](https://github.com/grafana/helm-charts/blob/28a1fb8338711e926c8a7419c525ea4e2d1f3ba5/charts/grafana/README.md#sidecar-for-datasources).

## Architecture

1. **Sealed Secret**: `grafana-timescaledb-secret-sealed.yaml` contains the database credentials (POSTGRES_USER, POSTGRES_PASSWORD)
2. **Datasource ConfigMap**: `grafana-timescaledb-datasource.yaml` contains the datasource configuration with environment variable placeholders
3. **HelmRelease**: Configures Grafana to:
   - Watch the datasource ConfigMap via `datasources.configmaps`
   - Mount credentials as environment variables via `envFrom` from the sealed secret
   - The sidecar automatically substitutes `#{VARIABLE_NAME}` placeholders

## How It Works

The Grafana sidecar:
- Watches ConfigMaps listed in `datasources.configmaps: [grafana-timescaledb-datasource]`
- Reads environment variables from the container
- Substitutes `#{POSTGRES_USER}` and `#{POSTGRES_PASSWORD}` in the datasource config
- Provisions the datasource in Grafana automatically

## Next Steps

1. **Apply the changes**:
   ```bash
   flux reconcile helmrelease grafana -n grafana
   ```

3. **Verify**:
   - Log into Grafana
   - Go to Configuration → Data Sources
   - You should see "TimescaleDB" datasource working
   - Run "Save & Test" to verify connectivity

## Benefits of This Approach

✅ **Dynamic Credentials**: Credentials are pulled from the sealed secret at runtime
✅ **Security**: No credentials stored in plaintext in Git
✅ **Automatic Provisioning**: Sidecar watches for changes and updates datasources
✅ **GitOps Friendly**: Can be sealed and committed to Git

## Troubleshooting

### "No Data" in dashboards
1. Check datasource is working: Configuration → Data Sources → TimescaleDB → "Save & Test"
2. Verify credentials are being substituted: Check sidecar logs
   ```bash
   kubectl logs -n grafana -l app.kubernetes.io/name=grafana --container=grafana-sc-datasources
   ```
3. Verify ConfigMap and Secret exist:
   ```bash
   kubectl get secret grafana-timescaledb-secret -n grafana
   kubectl get configmap grafana-timescaledb-datasource -n grafana
   ```

### Sidecar not provisioning datasource
- Check environment variables are mounted:
  ```bash
  kubectl exec -n grafana -l app.kubernetes.io/name=grafana -- env | grep POSTGRES
  ```
- Check sidecar is watching the correct secret:
  ```bash
  kubectl describe pod -n grafana -l app.kubernetes.io/name=grafana | grep -A5 Sidecar
  ```

