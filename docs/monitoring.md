# Monitoring

What watches the cluster, where alerts go, and how to investigate when something is
wrong.

## The stack

| Component | Namespace | Role |
|---|---|---|
| Prometheus (kube-prometheus-stack) | `monitoring` | Metrics from nodes, Kubernetes, Flux and Pi-hole. 15 days / 45 GB retention. |
| Alertmanager | `monitoring` | Routes Prometheus alerts to Slack |
| Loki + Alloy | `monitoring` | Alloy ships pod logs to Loki |
| Grafana | `grafana` | Dashboards at `grafana.k12n.com` |
| `monitoring-health-check` CronJob | `monitoring` | Pipeline checks that metrics do not cover (below) |

Prometheus picks up any `ServiceMonitor`, `PodMonitor` or `PrometheusRule` in any
namespace, so an app can ship its own next to its manifests.

## Alerts

Everything ends up in Slack, by two routes.

### Alertmanager

Prometheus rules fire through Alertmanager to the Slack receiver. Besides the default
rules that ship with kube-prometheus-stack, the repository defines:

| Alert | Fires when | Defined in |
|---|---|---|
| `NodeCPUTemperatureHigh` / `Critical` | A node's CPU is above 85 °C / 95 °C | `gitops/apps/monitoring/prometheusrule-node-thermal.yaml` |
| `GranbokBackupFailed` | A granbok backup job failed | `gitops/apps/granbok/prometheusrule-backup.yaml` |
| `GranbokBackupStale` | No successful granbok backup for 36 hours | same |
| `GranbokBackupNeverSucceeded` | The backup has never completed | same |

### Health-check CronJob

Every 15 minutes a script
(`gitops/apps/monitoring/monitoring-script-configmap.yaml`) checks the data pipeline:

| Check | Fires when | Severity |
|---|---|---|
| Energy freshness | No new energy reading for 10 minutes | critical |
| Heat pump freshness | No new heat pump reading for 10 minutes | warning |
| Temperature freshness | No new temperature reading for 2 hours | warning |
| Consumer lag | A consumer group is more than 1000 messages behind | warning |
| MQTT bridge | The bridge logged connection losses in the last 15 minutes | critical |
| Device commands | An outbox command was marked failed in the last 24 hours | critical |

It only posts on a change: once when a check starts failing, a reminder while it keeps
failing, and once when it recovers. Its state is kept in the `monitoring-health-state`
ConfigMap, which the script creates itself and which must stay out of Git.

### Dead man's switch

If the cluster is down, nothing inside it can send an alert. Two pings to
healthchecks.io cover that: Alertmanager forwards the always-firing `Watchdog` alert,
and the health-check script pings after every run. healthchecks.io notifies when the
pings stop.

## Dashboards

Dashboards are JSON files in `gitops/apps/grafana/dashboards/`, loaded into the
**Homelab** folder:

| Dashboard | Shows |
|---|---|
| Node Exporter Full | CPU, memory, disk, network and temperature per node |
| Kubernetes Global | Cluster-wide resource use |
| Flux Cluster / Flux Control Plane | Reconciliation status and controller health |
| Loki Logs | Log search across namespaces |
| Pi-hole | DNS queries and blocking |

To add one, put the JSON in that directory and add a `configMapGenerator` entry in
`gitops/apps/grafana/kustomization.yaml`.

## Investigating

### Is GitOps healthy?

```bash
flux get kustomizations
flux get helmreleases -A
```

### Is the pipeline flowing?

```bash
# Consumer lag per group
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group describe timescaledb-energy

# Bridge and sink logs
kubectl logs -n mqtt-kafka-bridge deployment/mqtt-kafka-bridge --tail=50
kubectl logs -n timescaledb deployment/redpanda-connect --tail=50
kubectl logs -n homelab-settings deployment/redpanda-connect --tail=50

# Redpanda itself
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk cluster health
```

[data-flow.md](data-flow.md) walks through following one reading end to end.

### What did the last health check say?

```bash
kubectl get jobs -n monitoring --sort-by=.status.startTime | tail -3
kubectl logs -n monitoring job/<job-name>
```

### Is a pod failing?

```bash
kubectl get pods -A | grep -v -E "Running|Completed"
kubectl describe pod -n <namespace> <pod>
kubectl logs -n <namespace> <pod> --previous
```

Logs older than a pod's lifetime are in Loki: open Grafana, choose **Explore**, select
the Loki data source and filter by namespace.

### Direct access

```bash
make port-prometheus    # http://localhost:9090
make port-grafana       # http://localhost:3000
```

## Not covered

- The TimescaleDB, homelab-settings, gasa and Backstage backup jobs have no alert. A
  failed run is visible only as a failed Job. See [BACKUP_SETUP.md](BACKUP_SETUP.md).
- Application services do not export their own Prometheus metrics; their health is
  inferred from the pipeline checks and Kubernetes probes.
