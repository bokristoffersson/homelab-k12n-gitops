# Dashboards

Dashboards are kept in Git as JSON and loaded automatically. They appear in the
**Homelab** folder.

## What is provisioned

| Dashboard | File | Data source | Shows |
|---|---|---|---|
| Node Exporter Full | `node-exporter-full.json` | Prometheus | CPU, memory, disk, network and temperatures per node |
| Kubernetes Global | `kubernetes-global.json` | Prometheus | Cluster-wide resource use by namespace and workload |
| Flux Cluster | `flux-cluster.json` | Prometheus | Reconciliation status of Kustomizations and HelmReleases |
| Flux Control Plane | `flux-control-plane.json` | Prometheus | Health of the Flux controllers |
| Loki Logs | `loki-logs.json` | Loki | Log search by namespace and pod |
| Pi-hole | `pihole.json` | Prometheus | DNS queries, blocking and clients |

The files are in `gitops/apps/grafana/dashboards/`.

## How loading works

`gitops/apps/grafana/kustomization.yaml` turns each JSON file into a ConfigMap with the
label `grafana_dashboard: "1"` and the annotation `grafana_folder: Homelab`. Grafana's
dashboard sidecar watches every namespace for that label and imports what it finds.

Because the sidecar looks in all namespaces, an app can also ship its own dashboard
ConfigMap next to its manifests.

## Adding a dashboard

1. Build it in the Grafana UI, or download one from grafana.com.
2. Export the JSON: **Share → Export → Save to file**, with "Export for sharing
   externally" off.
3. Replace data source references with the fixed UIDs `prometheus` or `loki`, so the
   dashboard does not depend on an import prompt.
4. Save the file under `gitops/apps/grafana/dashboards/`.
5. Add an entry to `kustomization.yaml`:

   ```yaml
   configMapGenerator:
     - name: grafana-dashboard-my-dashboard
       files:
         - dashboards/my-dashboard.json
       options:
         labels:
           grafana_dashboard: "1"
         annotations:
           grafana_folder: Homelab
   ```

6. Run `make lint`, open a PR and merge. The sidecar picks it up within a minute or two
   of Flux applying it; no restart is needed.

## Changing a dashboard

Edit it in the UI, export the JSON again and replace the file. Edits saved only in the
UI are overwritten the next time the sidecar reloads the ConfigMap.
