# Apache IoTDB Deployment for K3s with FluxCD

Simple IoTDB deployment without MQTT - access via RPC/SQL only.

## 📁 Directory Structure

```
iotdb/
├── namespace.yaml              # Creates the iotdb namespace
├── deployment.yaml             # Complete IoTDB deployment (ConfigNode + DataNode)
└── grafana-datasource.yaml     # Grafana datasource configuration
```

## 🏗️ Architecture

- **1 ConfigNode**: Manages cluster metadata
- **1 DataNode**: Handles data storage and queries
- Both pinned to `worker-pi5` node
- Uses `nvme-storage` StorageClass

## 🚀 Quick Deployment

```bash
# 1. Update your GitOps repo with the simplified manifest
cd /path/to/your/gitops-repo
# Copy the updated iotdb-helmrelease.yaml

# 2. Commit and push
git add iotdb/
git commit -m "Simplified IoTDB deployment - no MQTT"
git push

# 3. Reconcile FluxCD
flux reconcile source git flux-system
flux reconcile kustomization iotdb -n flux-system

# 4. Watch deployment
kubectl get pods -n iotdb -w
```

## 📊 Accessing IoTDB

### Via CLI

```bash
# Connect to IoTDB CLI
kubectl exec -it -n iotdb iotdb-datanode-0 -- \
  /iotdb/sbin/start-cli.sh -h localhost -u root -pw root

# Try some commands
IoTDB> show cluster
IoTDB> show databases
IoTDB> CREATE DATABASE root.test
IoTDB> INSERT INTO root.test.device1(timestamp,temperature) VALUES(now(), 25.5)
IoTDB> SELECT * FROM root.test.device1
```

### Via Port Forward (for external tools)

```bash
# Forward RPC port to your local machine
kubectl port-forward -n iotdb svc/iotdb-service 6667:6667
```

Then connect from your local machine using any IoTDB client to `localhost:6667`.

## 📈 Grafana Integration

The datasource ConfigMap is configured to connect to:
- **Host**: `iotdb-service.iotdb.svc.cluster.local:6667`
- **Database**: `root`
- **User**: `root`
- **Password**: `root`

### Query Examples

```sql
-- Get recent data
SELECT time, temperature FROM root.test.device1 WHERE time > now() - 1h

-- Aggregate data
SELECT avg(temperature) as avg_temp 
FROM root.test.device1 
WHERE time > now() - 24h 
GROUP BY time(5m)
```

## 🔧 Storage Configuration

- **ConfigNode**: 10Gi data + 5Gi logs
- **DataNode**: 50Gi data + 5Gi logs
- **StorageClass**: `nvme-storage`
- **Node**: Both on `worker-pi5`

## 🔍 Monitoring

```bash
# Check cluster status
kubectl exec -it -n iotdb iotdb-datanode-0 -- \
  /iotdb/sbin/start-cli.sh -h localhost -u root -pw root

IoTDB> show cluster
IoTDB> show datanodes

# Check logs
kubectl logs -n iotdb iotdb-confignode-0
kubectl logs -n iotdb iotdb-datanode-0

# Check resource usage
kubectl top pods -n iotdb
```

## 🔒 Security

**Default credentials**: `root:root`

To change:
```bash
kubectl exec -it -n iotdb iotdb-datanode-0 -- \
  /iotdb/sbin/start-cli.sh -h localhost -u root -pw root

IoTDB> ALTER USER root SET PASSWORD 'your_new_password';
IoTDB> CREATE USER myuser 'mypassword';
IoTDB> GRANT WRITE ON root.** TO USER myuser;
```

## 🐛 Troubleshooting

```bash
# Check pod status
kubectl get pods -n iotdb

# Check events
kubectl get events -n iotdb --sort-by='.lastTimestamp'

# Check PVCs
kubectl get pvc -n iotdb

# Describe problematic pods
kubectl describe pod -n iotdb <pod-name>

# View logs
kubectl logs -n iotdb <pod-name> --tail=100
```

## 🔄 Restart/Update

```bash
# Restart ConfigNode
kubectl delete pod -n iotdb iotdb-confignode-0

# Restart DataNode
kubectl delete pod -n iotdb iotdb-datanode-0

# Force FluxCD reconciliation
flux reconcile kustomization iotdb -n flux-system
```

## 📚 Resources

- [Apache IoTDB Documentation](https://iotdb.apache.org/)
- [IoTDB SQL Reference](https://iotdb.apache.org/UserGuide/latest/SQL-Manual/SQL-Manual.html)
- [IoTDB REST API](https://iotdb.apache.org/UserGuide/latest/API/RestServiceV2.html)