# Get Database Credentials

The Grafana helm chart doesn't support dynamic secret references in the datasource configuration, so you need to get the actual values and update the `helmrelease.yaml` file.

## Quick Steps

1. **Get the credentials**:
   ```bash
   kubectl get secret timescaledb-secret -n heatpump-mqtt -o jsonpath='{.data.POSTGRES_USER}' | base64 -d && echo
   kubectl get secret timescaledb-secret -n heatpump-mqtt -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d && echo
   ```

2. **Update apps/base/grafana/helmrelease.yaml**:
   - Line 53: Replace `YOUR_DB_USER_HERE` with the username from step 1
   - Line 55: Replace `YOUR_DB_PASSWORD_HERE` with the password from step 1

3. **Commit and push**:
   ```bash
   git add apps/base/grafana/helmrelease.yaml
   git commit -m "Update Grafana TimescaleDB datasource credentials"
   git push
   ```

4. **Reconcile**:
   ```bash
   flux reconcile helmrelease grafana -n grafana
   ```

## Alternative: Use Your Grafana Secrets

If you want to use the values from the `grafana-timescaledb-secret` you created, you can get them after Flux deploys:

```bash
kubectl get secret grafana-timescaledb-secret -n grafana -o jsonpath='{.data.POSTGRES_USER}' | base64 -d && echo
kubectl get secret grafana-timescaledb-secret -n grafana -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d && echo
```

Then update the helmrelease.yaml with those values.



