#!/bin/bash
# Debug script to check runner pod status and logs

echo "=== Checking Runner Pods ==="
kubectl get pods -n actions-runners

echo -e "\n=== Checking Ephemeral Runners ==="
kubectl get ephemeralrunner -n actions-runners

echo -e "\n=== Recent Runner Events ==="
kubectl get events -n actions-runners --sort-by='.lastTimestamp' | tail -20

echo -e "\n=== Checking for Terminated Pods ==="
for pod in $(kubectl get pods -n actions-runners -o jsonpath='{.items[*].metadata.name}'); do
  status=$(kubectl get pod $pod -n actions-runners -o jsonpath='{.status.containerStatuses[0].state.terminated.reason}' 2>/dev/null)
  if [ ! -z "$status" ]; then
    echo "Pod: $pod - Terminated reason: $status"
    exit_code=$(kubectl get pod $pod -n actions-runners -o jsonpath='{.status.containerStatuses[0].state.terminated.exitCode}' 2>/dev/null)
    echo "  Exit code: $exit_code"
    echo "  Last logs:"
    kubectl logs $pod -n actions-runners --tail=20 2>&1 | tail -5
  fi
done

echo -e "\n=== Current Running Pod Logs (last 30 lines) ==="
for pod in $(kubectl get pods -n actions-runners --field-selector=status.phase=Running -o jsonpath='{.items[*].metadata.name}'); do
  echo "--- Pod: $pod ---"
  kubectl logs $pod -n actions-runners --tail=30 2>&1 | tail -10
done

