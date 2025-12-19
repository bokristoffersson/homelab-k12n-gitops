#!/usr/bin/env bash
set -euo pipefail

POD="redpanda-v2-0"
NAMESPACE="redpanda-v2"

echo "Creating Redpanda topics..."

# Function to create topic if it doesn't exist
create_topic() {
    local topic=$1
    local partitions=${2:-3}
    local retention=${3:-604800000}  # 7 days in ms

    echo "Creating topic: $topic (partitions=$partitions, retention=${retention}ms)"

    # Check if topic exists
    if kubectl exec -n $NAMESPACE $POD -- rpk topic list | grep -q "^$topic"; then
        echo "  ✓ Topic $topic already exists"
    else
        kubectl exec -n $NAMESPACE $POD -- \
            rpk topic create "$topic" \
            --partitions "$partitions" \
            --replicas 1 \
            --topic-config retention.ms="$retention"
        echo "  ✓ Created topic $topic"
    fi
}

# Wait for Redpanda to be ready
echo "Waiting for Redpanda to be ready..."
kubectl wait --for=condition=Ready pod/$POD -n $NAMESPACE --timeout=120s

echo ""
echo "Creating topics..."

# Energy data (high frequency, shorter retention)
create_topic "homelab-energy-realtime" 3 86400000  # 1 day

# Temperature data (lower frequency, longer retention)
create_topic "homelab-temperature-indoor" 3 604800000   # 7 days
create_topic "homelab-temperature-outdoor" 3 604800000  # 7 days

# Heatpump data
create_topic "homelab-heatpump-status" 3 604800000      # 7 days
create_topic "homelab-heatpump-telemetry" 3 604800000   # 7 days
create_topic "homelab-heatpump-settings" 3 604800000    # 7 days
create_topic "homelab-heatpump-realtime" 3 86400000     # 1 day

# Sensor state
create_topic "homelab-sensor-state" 3 604800000         # 7 days

echo ""
echo "✅ Topics created successfully!"
echo ""
echo "List topics:"
kubectl exec -n $NAMESPACE $POD -- rpk topic list
