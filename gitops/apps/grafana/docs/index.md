# Grafana

Dashboards for the cluster's infrastructure, at `https://grafana.k12n.com`. Grafana runs
in the `grafana` namespace from the Grafana Helm chart, with one replica and a Longhorn
volume.

It is for operating the cluster: nodes, Kubernetes, Flux, logs and DNS. The household
data (energy, heat pump, temperature) is shown by
[heatpump-web](../../../applications/heatpump-web), not here.

## Data sources

Data sources are ConfigMaps labelled `grafana_datasource`, loaded by a sidecar.

| Name | UID | Points at |
|---|---|---|
| Prometheus | `prometheus` | `kube-prometheus-stack-prometheus` in `monitoring` |
| Loki | `loki` | `loki` in `monitoring` |
| TimescaleDB | `timescaledb` | See the note below |

**The TimescaleDB data source does not work.** The Job that generates it writes a URL
in the `heatpump-mqtt` namespace, which no longer exists, and its secret names a
database that is not `telemetry`. No provisioned dashboard uses it.

## Dashboards

See [Dashboards](dashboards.md) for the list and how to add one.

## Login

Grafana uses its own login; it is not behind oauth2-proxy. The admin user and password
come from the `grafana-admin` secret.

```bash
kubectl get secret grafana-admin -n grafana -o jsonpath='{.data.admin-user}' | base64 -d
```

## Operations

```bash
kubectl get pods -n grafana
kubectl logs -n grafana deployment/grafana -c grafana --tail=50
flux get helmrelease grafana -n grafana
make port-grafana                       # http://localhost:3000
```

| Symptom | Check |
|---|---|
| A dashboard is missing | The sidecar log: `kubectl logs -n grafana deployment/grafana -c grafana-sc-dashboard` |
| Panels show "No data" | The data source under **Connections**, then whether Prometheus or Loki is up in `monitoring` |
| Changes made in the UI disappear | Provisioned dashboards are reloaded from Git. Change the JSON file instead. |

## Related

- [Monitoring](https://github.com/bokristoffersson/homelab-k12n-gitops/blob/main/docs/monitoring.md): alerts and how the pieces fit
- Prometheus, Loki and Alloy: `gitops/apps/monitoring`, `gitops/apps/loki`, `gitops/apps/alloy`
