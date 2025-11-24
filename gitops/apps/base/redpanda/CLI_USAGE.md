# Redpanda CLI Usage Guide

This guide shows how to use the Redpanda CLI (`rpk`) from your developer machine or on the k3s node.

## ⚠️ kubectl Exec Fails for Redpanda But Works for Other Pods?

**If `kubectl exec` works for other pods but fails for Redpanda**, this is likely because:

1. **Redpanda pod has multiple containers** - You need to specify the container name
2. **Container entrypoint** - The Redpanda container runs `rpk redpanda start` as PID 1
3. **Default container** - kubectl might be trying the wrong container

### Solution: Always Specify the Container

The Redpanda pod has two containers: `redpanda` and `sidecar`. Always use `-c redpanda`:

```bash
# ✅ Correct - specify container
kubectl exec redpanda-0 -n redpanda -c redpanda -- rpk topic list

# ❌ Wrong - might try sidecar container or fail
kubectl exec redpanda-0 -n redpanda -- rpk topic list
```

### If You Still Get Proxy Errors

**If you get "proxy error" even with `-c redpanda`**, this means your developer machine can't connect to the k3s cluster. Use one of these solutions:

### Quick Fix: Access from k3s Node Directly (Recommended)

SSH into your k3s node and run commands there:
```bash
# SSH into the k3s node (192.168.50.210)
ssh user@192.168.50.210

# Then on the node, run:
kubectl exec redpanda-0 -n redpanda -c redpanda -- rpk topic list
```

### Alternative: Install rpk on k3s Node

If you're on the k3s node, you can use rpk directly:
```bash
# Install rpk on the node
curl -1sLf 'https://dl.redpanda.com/nzc4ZYQK3WRGd9c/redpanda/cfg/setup/bash.deb.sh' | sudo -E bash
sudo apt-get install redpanda -y

# Use service DNS (works from within the cluster)
rpk topic list --brokers redpanda.redpanda.svc.cluster.local:9092
```

See [Troubleshooting](#troubleshooting) section below for more solutions.

## Option 1: Using kubectl exec (Recommended - Works Everywhere)

This is the simplest method and works from both your developer machine and the k3s node.

### List Topics
```bash
kubectl exec -it redpanda-0 -n redpanda -- rpk topic list
```

### Describe a Topic
```bash
kubectl exec -it redpanda-0 -n redpanda -- rpk topic describe heatpump-realtime
kubectl exec -it redpanda-0 -n redpanda -- rpk topic describe energy-realtime
```

### Get Cluster Info
```bash
kubectl exec -it redpanda-0 -n redpanda -- rpk cluster info
```

### Create a Topic
```bash
kubectl exec -it redpanda-0 -n redpanda -- rpk topic create my-topic \
  --partitions 1 \
  --replicas 1 \
  -c retention.ms=3600000
```

### Delete a Topic
```bash
kubectl exec -it redpanda-0 -n redpanda -- rpk topic delete my-topic
```

### Produce Messages
```bash
# Interactive mode
kubectl exec -it redpanda-0 -n redpanda -- rpk topic produce heatpump-realtime \
  --key "device-001"

# Non-interactive (pipe input)
echo '{"device_id":"device-001","timestamp":1698765432,"d0":5.2,"d5":35.0,"d6":30.5}' | \
  kubectl exec -i redpanda-0 -n redpanda -- rpk topic produce heatpump-realtime \
  --key "device-001"
```

### Consume Messages
```bash
# Consume last 10 messages
kubectl exec -it redpanda-0 -n redpanda -- rpk topic consume heatpump-realtime \
  --format '%k: %v\n' \
  --num 10

# Consume from beginning
kubectl exec -it redpanda-0 -n redpanda -- rpk topic consume heatpump-realtime \
  --format '%k: %v\n' \
  --offset 0
```

### List Consumer Groups
```bash
kubectl exec -it redpanda-0 -n redpanda -- rpk group list
```

### Describe Consumer Group
```bash
kubectl exec -it redpanda-0 -n redpanda -- rpk group describe my-group
```

## Option 2: Port Forward + Local rpk (Developer Machine Only)

If you want to use `rpk` directly on your developer machine, you need to:
1. Install `rpk` locally
2. Port-forward the Redpanda service
3. Connect using the forwarded port

### Step 1: Install rpk on Your Developer Machine

**macOS:**
```bash
brew install redpanda-data/tap/redpanda
```

**Linux:**
```bash
curl -1sLf 'https://dl.redpanda.com/nzc4ZYQK3WRGd9c/redpanda/cfg/setup/bash.deb.sh' | sudo -E bash
sudo apt-get install redpanda -y
```

**Or download binary:**
```bash
# Check latest version at https://github.com/redpanda-data/redpanda/releases
wget https://github.com/redpanda-data/redpanda/releases/download/v24.2.19/rpk-darwin-arm64.zip
unzip rpk-darwin-arm64.zip
sudo mv rpk /usr/local/bin/
```

### Step 2: Port Forward Redpanda Service

In a separate terminal, keep this running:
```bash
kubectl port-forward -n redpanda svc/redpanda 9092:9092
```

### Step 3: Use rpk with Localhost

```bash
# List topics
rpk topic list --brokers localhost:9092

# Describe topic
rpk topic describe heatpump-realtime --brokers localhost:9092

# Cluster info
rpk cluster info --brokers localhost:9092

# Produce message
echo '{"test": "data"}' | rpk topic produce heatpump-realtime \
  --brokers localhost:9092 \
  --key "test-key"

# Consume messages
rpk topic consume heatpump-realtime \
  --brokers localhost:9092 \
  --format '%k: %v\n' \
  --num 10
```

## Option 3: On k3s Node Directly

If you're SSH'd into the k3s node, you can:

### Option 3a: Use kubectl exec (Same as Option 1)
```bash
kubectl exec -it redpanda-0 -n redpanda -- rpk topic list
```

### Option 3b: Install rpk on the Node and Connect to Service

```bash
# Install rpk on the node
curl -1sLf 'https://dl.redpanda.com/nzc4ZYQK3WRGd9c/redpanda/cfg/setup/bash.deb.sh' | sudo -E bash
sudo apt-get install redpanda -y

# Use the service DNS name
rpk topic list --brokers redpanda.redpanda.svc.cluster.local:9092

# Or use the pod IP directly
POD_IP=$(kubectl get pod redpanda-0 -n redpanda -o jsonpath='{.status.podIP}')
rpk topic list --brokers ${POD_IP}:9092
```

## Quick Reference Commands

### Most Common Operations

```bash
# List all topics
kubectl exec -it redpanda-0 -n redpanda -- rpk topic list

# Describe a specific topic
kubectl exec -it redpanda-0 -n redpanda -- rpk topic describe heatpump-realtime

# Get cluster information
kubectl exec -it redpanda-0 -n redpanda -- rpk cluster info

# Produce a test message
echo '{"test": "message"}' | kubectl exec -i redpanda-0 -n redpanda -- \
  rpk topic produce heatpump-realtime --key "test"

# Consume messages
kubectl exec -it redpanda-0 -n redpanda -- rpk topic consume heatpump-realtime \
  --format '%k: %v\n' --num 5
```

## Creating Helper Scripts

You can create helper scripts to make this easier:

### Create `redpanda-list-topics.sh`
```bash
#!/bin/bash
kubectl exec -it redpanda-0 -n redpanda -- rpk topic list
```

### Create `redpanda-describe-topic.sh`
```bash
#!/bin/bash
TOPIC=${1:-heatpump-realtime}
kubectl exec -it redpanda-0 -n redpanda -- rpk topic describe $TOPIC
```

### Create `redpanda-produce.sh`
```bash
#!/bin/bash
TOPIC=${1:-heatpump-realtime}
KEY=${2:-default-key}
kubectl exec -i redpanda-0 -n redpanda -- rpk topic produce $TOPIC --key "$KEY"
```

Usage:
```bash
chmod +x redpanda-*.sh
./redpanda-list-topics.sh
./redpanda-describe-topic.sh energy-realtime
echo '{"data": "test"}' | ./redpanda-produce.sh heatpump-realtime device-001
```

## Troubleshooting

### Why kubectl exec Works for Other Pods But Not Redpanda

**The Redpanda pod has 2 containers: `redpanda` and `sidecar`**

When you don't specify `-c redpanda`, kubectl might:
- Try to exec into the wrong container (sidecar)
- Fail because the sidecar container doesn't have `rpk`
- Default to a container that has different exec restrictions

**Always specify the container:**
```bash
# ✅ Always use -c redpanda
kubectl exec redpanda-0 -n redpanda -c redpanda -- rpk topic list

# Check which containers exist
kubectl get pod redpanda-0 -n redpanda -o jsonpath='{.spec.containers[*].name}'
# Output: redpanda sidecar
```

**If you still get proxy errors even with `-c redpanda`**, try these diagnostics:

```bash
# 1. Check which node the pod is on
kubectl get pod redpanda-0 -n redpanda -o wide

# 2. Compare with a working pod
kubectl get pods -A -o wide | grep -E "(NAME|redpanda-0|your-working-pod)"

# 3. Try exec into the sidecar container (to test if it's container-specific)
kubectl exec redpanda-0 -n redpanda -c sidecar -- echo "test"

# 4. Check if it's a node-specific issue
kubectl get nodes
kubectl describe node <node-name> | grep -i "conditions\|ready"

# 5. Try using the pod's shell directly
kubectl exec redpanda-0 -n redpanda -c redpanda -- sh -c "echo test"
# If sh doesn't work, try:
kubectl exec redpanda-0 -n redpanda -c redpanda -- /bin/sh -c "echo test"
```

**Common causes:**
- **Different node**: Redpanda pod might be on a node with network/firewall issues
- **Container runtime**: The container might be using a different runtime (containerd vs docker)
- **Resource constraints**: Node might be under heavy load
- **Network policies**: Namespace might have network policies affecting exec

### Port Forward Error: "proxy error from 127.0.0.1:6443"

If you see this error:
```
error: error upgrading connection: error dialing backend: proxy error from 127.0.0.1:6443 while dialing 192.168.50.210:10250, code 502: 502 Bad Gateway
```

**This means kubectl can't connect to the k3s node.** Solutions:

#### Solution 1: Access from k3s Node Directly (When kubectl exec Also Fails)

If both `kubectl exec` and `port-forward` fail with proxy errors, the issue is network connectivity. **SSH into the k3s node** and run commands there:

```bash
# SSH into your k3s node
ssh user@192.168.50.210  # Replace with your k3s node IP/user

# On the node, kubectl exec should work
kubectl exec redpanda-0 -n redpanda -c redpanda -- rpk topic list
kubectl exec redpanda-0 -n redpanda -c redpanda -- rpk cluster info
```

**Or install rpk on the node and use service DNS:**
```bash
# On the k3s node, install rpk
curl -1sLf 'https://dl.redpanda.com/nzc4ZYQK3WRGd9c/redpanda/cfg/setup/bash.deb.sh' | sudo -E bash
sudo apt-get install redpanda -y

# Use service DNS (works from within cluster)
rpk topic list --brokers redpanda.redpanda.svc.cluster.local:9092
rpk topic describe heatpump-realtime --brokers redpanda.redpanda.svc.cluster.local:9092
```

#### Solution 2: Fix kubectl Proxy/Network Issues

If you need kubectl to work from your developer machine:
```bash
# Test if you can reach the k3s API server
kubectl cluster-info

# Check if the node is reachable
ping 192.168.50.210

# Verify kubectl config
kubectl config view
```

#### Solution 3: Access from k3s Node Directly
SSH into the k3s node and run commands there:
```bash
# On the k3s node
kubectl exec redpanda-0 -n redpanda -c redpanda -- rpk topic list

# Or install rpk on the node and use service DNS
rpk topic list --brokers redpanda.redpanda.svc.cluster.local:9092
```

#### Solution 4: Fix Network/Proxy Issues on Developer Machine

If both `kubectl exec` and `port-forward` fail with "proxy error", try these fixes:

**Check and disable proxy:**
```bash
# Check if proxy is set
echo $HTTP_PROXY
echo $HTTPS_PROXY
env | grep -i proxy

# Temporarily disable proxy for kubectl
unset HTTP_PROXY HTTPS_PROXY http_proxy https_proxy NO_PROXY no_proxy
kubectl exec redpanda-0 -n redpanda -c redpanda -- rpk topic list
```

**Check kubectl configuration:**
```bash
# View current context and config
kubectl config current-context
kubectl config view

# Test cluster connectivity
kubectl cluster-info
kubectl get nodes

# If using k3s, check if kubeconfig server address is correct
cat ~/.kube/config | grep server
# Should point to your k3s node IP, not 127.0.0.1
```

**Check network connectivity:**
```bash
# Test if you can reach the k3s node
ping 192.168.50.210

# Test if kubelet port is accessible (may be blocked by firewall)
nc -zv 192.168.50.210 10250
# or
telnet 192.168.50.210 10250
```

**Fix k3s kubeconfig (if server is 127.0.0.1):**
```bash
# On the k3s node, get the kubeconfig
sudo cat /etc/rancher/k3s/k3s.yaml

# On your developer machine, update ~/.kube/config
# Replace "127.0.0.1" or "localhost" with the actual k3s node IP (192.168.50.210)
sed -i '' 's/127.0.0.1/192.168.50.210/g' ~/.kube/config
# or manually edit ~/.kube/config
```

#### Solution 5: Use NodePort Service (For External Access)
If you need external access, create a NodePort service:
```yaml
apiVersion: v1
kind: Service
metadata:
  name: redpanda-nodeport
  namespace: redpanda
spec:
  type: NodePort
  selector:
    app.kubernetes.io/name: redpanda
  ports:
  - port: 9092
    targetPort: 9092
    nodePort: 30092
```

Then connect to `<k3s-node-ip>:30092` from anywhere on your network.

### Connection Refused
- Make sure Redpanda pod is running: `kubectl get pods -n redpanda`
- Check service: `kubectl get svc -n redpanda`
- Verify pod is ready: `kubectl get pod redpanda-0 -n redpanda`

### Command Not Found (rpk)
- If using kubectl exec, rpk is already in the container
- If installing locally, verify installation: `rpk --version`
- Make sure you're using the correct container: `-c redpanda`

### Port Already in Use
- Check if port 9092 is already in use: `lsof -i :9092`
- Kill the existing port-forward: `pkill -f "port-forward.*9092"`
- Use a different local port: `kubectl port-forward -n redpanda svc/redpanda 9093:9092`

## Advanced: Using Redpanda Console

Redpanda Console is already deployed! Access it via port-forward:

```bash
# Port forward the console
kubectl port-forward -n redpanda svc/redpanda-console 8080:8080

# Then open http://localhost:8080 in your browser
```

The console provides a web UI for:
- Viewing topics
- Producing/consuming messages
- Monitoring cluster health
- Viewing consumer groups

