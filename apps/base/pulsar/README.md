# Apache Pulsar GitOps Deployment

This setup deploys Apache Pulsar to your k3s cluster using FluxCD with persistent storage on your worker-pi5 node.

## Directory Structure

```
your-flux-repo/
├── infrastructure/
│   └── pulsar/
│       ├── namespace.yaml          # Namespace and HelmRepository
│       ├── helmrelease.yaml        # HelmRelease with values
│       └── kustomization.yaml      # Kustomization (optional, or add to higher level)
└── clusters/
    └── home/
        └── infrastructure.yaml     # Reference to pulsar kustomization
```

## Deployment Steps

### 1. Create the Pulsar Directory

```bash
mkdir -p infrastructure/pulsar
```

### 2. Add the Files

Copy the provided YAML files:
- `namespace.yaml` - Contains namespace and HelmRepository
- `helmrelease.yaml` - Contains HelmRelease with Pulsar configuration
- `kustomization.yaml` - (Optional) FluxCD Kustomization resource

### 3. Commit and Push

```bash
git add infrastructure/pulsar/
git commit -m "Add Apache Pulsar deployment"
git push
```

### 4. Let Flux Reconcile (or force it)

```bash
# Force reconciliation
flux reconcile source git flux-system
flux reconcile kustomization flux-system

# Watch the deployment
kubectl get helmrelease -n pulsar -w
```

### 5. Verify Deployment

```bash
# Check all pods are running
kubectl get pods -n pulsar

# Check services
kubectl get svc -n pulsar

# Check PVCs are bound
kubectl get pvc -n pulsar
```

Expected pods:
- `pulsar-zookeeper-0` - Metadata storage
- `pulsar-bookkeeper-0` - Message storage
- `pulsar-broker-*` - Message broker
- `pulsar-proxy-*` - Client proxy

## Accessing Pulsar

### Internal Access (from within cluster)

```bash
# Pulsar binary protocol
pulsar://pulsar-proxy.pulsar.svc.cluster.local:6650

# HTTP admin API
http://pulsar-proxy.pulsar.svc.cluster.local:8080

# WebSocket
ws://pulsar-proxy.pulsar.svc.cluster.local:8000
```

### External Access

The proxy service is exposed as LoadBalancer. Get the external IP:

```bash
kubectl get svc -n pulsar pulsar-proxy
```

For your use case:
- **WebSocket (mobile/web app)**: `ws://<EXTERNAL-IP>:8000`
- **Admin API**: `http://<EXTERNAL-IP>:8080`
- **Pulsar protocol**: `pulsar://<EXTERNAL-IP>:6650`

## Configuration Details

### Resource Allocation

Current configuration (single-node homelab optimized):
- **Zookeeper**: 512Mi RAM, 250m CPU, 5Gi storage
- **BookKeeper**: 1Gi RAM, 500m CPU, 30Gi storage (10Gi journal + 20Gi ledgers)
- **Broker**: 1Gi RAM, 500m CPU
- **Proxy**: 512Mi RAM, 250m CPU

**Total**: ~3Gi RAM, ~1.5 CPU cores, ~35Gi NVMe storage

### Storage Classes

All components use `nvme-storage` StorageClass:
- Zookeeper data: 5Gi
- BookKeeper journal: 10Gi
- BookKeeper ledgers: 20Gi

### Node Affinity

All pods are pinned to `worker-pi5` using node affinity.

## Next Steps

### 1. Install Pulsar Admin Tools

```bash
# Create a pod for admin tasks
kubectl run -it --rm pulsar-admin \
  --image=apachepulsar/pulsar:latest \
  --restart=Never \
  --namespace=pulsar \
  -- /bin/bash

# Inside the pod
bin/pulsar-admin --admin-url http://pulsar-proxy:8080 tenants list
```

### 2. Create Tenants, Namespaces, and Topics

```bash
# Create tenant for IoT
bin/pulsar-admin tenants create iot

# Create namespace
bin/pulsar-admin namespaces create iot/sensors

# Set retention policy (e.g., 7 days)
bin/pulsar-admin namespaces set-retention iot/sensors --size 10G --time 7d

# Create topics for your devices
bin/pulsar-admin topics create persistent://iot/sensors/heatpump
bin/pulsar-admin topics create persistent://iot/sensors/temperature
bin/pulsar-admin topics create persistent://iot/sensors/power
```

### 3. MQTT Integration

For MQTT device integration, you have two options:

**Option A**: Use Pulsar's MQTT Protocol Handler (recommended)
- Requires enabling the MQTT protocol handler in broker config
- MQTT devices connect directly to Pulsar
- Native protocol conversion

**Option B**: Use MQTT Proxy/Bridge
- Deploy Mosquitto or HiveMQ with Pulsar connector
- MQTT devices → Mosquitto → Pulsar
- More flexible for legacy MQTT devices

### 4. WebSocket Client Connection

For your mobile/web app, use Pulsar's WebSocket API:

```javascript
const ws = new WebSocket('ws://pulsar-proxy-ip:8000/ws/v2/consumer/persistent/iot/sensors/heatpump/my-subscription');

ws.onmessage = function(event) {
  const data = JSON.parse(event.data);
  // Handle real-time sensor data
};
```

### 5. IoTDB Integration

To stream data to IoTDB:
- Create Pulsar Functions or use Pulsar IO connector
- Subscribe to sensor topics
- Transform and write to IoTDB time series

## Scaling Considerations

### For Production or Growth

If you add more nodes or need higher availability:

```yaml
# Increase replicas
zookeeper:
  replicaCount: 3  # Odd number for quorum

bookkeeper:
  replicaCount: 3  # For data replication

broker:
  replicaCount: 2  # For load balancing
```

Remove or relax node affinity to allow scheduling on multiple nodes.

### Current Single-Node Setup

The current configuration is optimized for:
- Homelab/development use
- Single worker node (worker-pi5)
- IoT sensor data ingestion
- Moderate message throughput
- 7-30 day retention

## Monitoring

To enable monitoring later:

```yaml
monitoring:
  prometheus: true
  grafana: true
```

This will deploy Prometheus and Grafana with pre-configured Pulsar dashboards.

## Troubleshooting

### Pods not starting

```bash
# Check pod events
kubectl describe pod <pod-name> -n pulsar

# Check logs
kubectl logs <pod-name> -n pulsar
```

### Storage issues

```bash
# Check PVC status
kubectl get pvc -n pulsar

# Check PV binding
kubectl get pv
```

### Common issues

1. **PVCs stuck in Pending**: Verify `nvme-storage` StorageClass exists
2. **Pods stuck in Pending**: Check node affinity and available resources on worker-pi5
3. **BookKeeper won't start**: Ensure journal and ledger PVCs are bound
4. **Connection timeout**: Verify service is LoadBalancer and has external IP

## Useful Commands

```bash
# Watch all resources
kubectl get all -n pulsar -w

# Force Flux reconciliation
flux reconcile helmrelease pulsar -n pulsar

# Get service endpoints
kubectl get endpoints -n pulsar

# Port forward for local testing
kubectl port-forward -n pulsar svc/pulsar-proxy 8080:8080 6650:6650 8000:8000
```

## Resources

- [Pulsar Documentation](https://pulsar.apache.org/docs/)
- [Pulsar Helm Chart](https://github.com/apache/pulsar-helm-chart)
- [WebSocket API](https://pulsar.apache.org/docs/client-libraries-websocket/)
- [MQTT on Pulsar](https://github.com/streamnative/mop)