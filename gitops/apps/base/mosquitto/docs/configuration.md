# Configuration

## Deployment Configuration

Mosquitto is deployed via Kubernetes manifests in the `gitops/apps/base/mosquitto` directory.

### Deployment Spec

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mosquitto
  namespace: mosquitto
spec:
  replicas: 1  # Single broker instance
  selector:
    matchLabels:
      app: mosquitto
  template:
    spec:
      containers:
      - name: mosquitto
        image: eclipse-mosquitto:2.0
        ports:
        - containerPort: 1883
          name: mqtt
          protocol: TCP
        - containerPort: 9001
          name: websocket
          protocol: TCP
        volumeMounts:
        - name: config
          mountPath: /mosquitto/config
        - name: data
          mountPath: /mosquitto/data
        resources:
          requests:
            cpu: 50m
            memory: 64Mi
          limits:
            cpu: 200m
            memory: 128Mi
      volumes:
      - name: config
        configMap:
          name: mosquitto-config
      - name: data
        persistentVolumeClaim:
          claimName: mosquitto-data
```

## Mosquitto Configuration

The broker configuration is managed via ConfigMap.

### mosquitto.conf

```conf
# Network settings
listener 1883
protocol mqtt

listener 9001
protocol websockets

# Allow anonymous connections (internal cluster only)
allow_anonymous true

# Persistence
persistence true
persistence_location /mosquitto/data/

# Logging
log_dest stdout
log_type error
log_type warning
log_type notice
log_type information
log_timestamp true

# Connection limits
max_connections -1
max_queued_messages 1000
message_size_limit 0

# Performance tuning
autosave_interval 300
autosave_on_changes false
```

### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: mosquitto-config
  namespace: mosquitto
data:
  mosquitto.conf: |
    listener 1883
    protocol mqtt
    allow_anonymous true
    persistence true
    persistence_location /mosquitto/data/
    log_dest stdout
    max_queued_messages 1000
```

## Storage

### Persistent Volume

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: mosquitto-data
  namespace: mosquitto
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 1Gi
  storageClassName: local-path  # k3s default
```

Stores:
- Retained messages
- Subscription data
- Persistent sessions

## Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: mosquitto
  namespace: mosquitto
spec:
  type: ClusterIP
  ports:
  - port: 1883
    targetPort: 1883
    protocol: TCP
    name: mqtt
  selector:
    app: mosquitto
```

## Environment Variables

No environment variables are required. All configuration is done via `mosquitto.conf`.

## Resource Limits

- **CPU Request**: 50m (0.05 cores)
- **CPU Limit**: 200m (0.2 cores)
- **Memory Request**: 64Mi
- **Memory Limit**: 128Mi

Sufficient for typical homelab workload (10-20 messages/second).

## Scaling Considerations

Mosquitto runs as a single instance because:
1. Message volume is low (<100 msg/s)
2. Clustering adds complexity without benefit at this scale
3. Persistent storage is single-writer (RWO)

For high availability, consider:
- Running multiple independent brokers
- Using MQTT bridge mode for replication
- Migrating to clustered broker (e.g., VerneMQ, EMQX)

## Security Configuration

### Current (Development)
```conf
allow_anonymous true
# No TLS
# No authentication
```

### Production Recommendations
```conf
# Require authentication
allow_anonymous false
password_file /mosquitto/config/passwd

# Enable TLS
listener 8883
protocol mqtt
cafile /mosquitto/certs/ca.crt
certfile /mosquitto/certs/server.crt
keyfile /mosquitto/certs/server.key
require_certificate false
```

Generate password file:
```bash
mosquitto_passwd -c /mosquitto/config/passwd username
```

## Client Configuration

### Device Configuration

Shelly devices connect to:
- **Host**: `mosquitto.mosquitto.svc.cluster.local`
- **Port**: 1883
- **Protocol**: MQTT 3.1.1
- **Auth**: None (anonymous)

### mqtt-kafka-bridge Configuration

```yaml
mqtt:
  broker: tcp://mosquitto.mosquitto:1883
  topics:
    - shelly+/events/rpc
  qos: 1
  client_id: mqtt-kafka-bridge
```

## Monitoring Configuration

### Prometheus Exporter

Mosquitto doesn't expose Prometheus metrics natively. For monitoring, consider:

1. **mosquitto-exporter**:
   ```bash
   kubectl apply -f mosquitto-exporter-deployment.yaml
   ```

2. **Monitor via logs**:
   ```bash
   kubectl logs -n mosquitto -l app=mosquitto -f
   ```

3. **Check connection count**:
   Use `$SYS` topics:
   ```bash
   mosquitto_sub -h localhost -t '$SYS/broker/clients/connected'
   ```

## Backup

### Configuration Backup
ConfigMaps are stored in Git (GitOps), no manual backup needed.

### Data Backup

```bash
# Export persistent volume
kubectl exec -n mosquitto mosquitto-<pod> -- \
  tar czf - /mosquitto/data | \
  kubectl cp mosquitto-<pod>:/mosquitto/data - > mosquitto-data-backup.tar.gz
```

### Restore

```bash
kubectl cp mosquitto-data-backup.tar.gz \
  mosquitto/<pod>:/tmp/backup.tar.gz

kubectl exec -n mosquitto mosquitto-<pod> -- \
  tar xzf /tmp/backup.tar.gz -C /
```

## Upgrading

Update image version in deployment:

```yaml
image: eclipse-mosquitto:2.0  # -> 2.1
```

FluxCD will automatically apply the change.

**Note**: Always check release notes for breaking changes in mosquitto.conf format.

## Performance Tuning

### High Message Rate

```conf
max_queued_messages 10000    # Increase buffer
autosave_interval 60         # More frequent saves
autosave_on_changes false    # Batch writes
```

### High Client Count

```conf
max_connections 500          # Allow more clients
sys_interval 60              # Reduce $SYS topic updates
```

### Low Latency

```conf
autosave_on_changes true     # Immediate persistence
max_queued_messages 100      # Smaller buffer
```

## Troubleshooting

### Broker Not Starting

Check logs:
```bash
kubectl logs -n mosquitto -l app=mosquitto
```

Common issues:
- Config syntax error: Validate mosquitto.conf
- Port conflict: Ensure 1883 is available
- Volume mount issue: Check PVC status

### Clients Can't Connect

1. Verify service DNS:
   ```bash
   nslookup mosquitto.mosquitto.svc.cluster.local
   ```

2. Test connectivity:
   ```bash
   kubectl port-forward -n mosquitto svc/mosquitto 1883:1883
   mosquitto_sub -h localhost -t '#'
   ```

3. Check firewall/network policies

### Message Loss

1. Increase QoS level (device → broker → subscriber all need matching QoS)
2. Enable persistence (already enabled)
3. Check `max_queued_messages` isn't too low

### High Memory Usage

1. Reduce `max_queued_messages`
2. Set `message_size_limit`
3. Disable unnecessary logging
4. Clear retained messages:
   ```bash
   mosquitto_sub -h localhost -t '#' --remove-retained
   ```
