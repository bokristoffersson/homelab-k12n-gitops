# Redpanda Sink Setup Guide

This guide explains how to manually create the required secrets for the redpanda-sink application.

## Required Secrets

The application requires the following secrets:

1. **timescaledb-secret** - Database connection credentials
2. **ghcr-secret** - GitHub Container Registry authentication (already provided)

## Creating the TimescaleDB Secret

The `timescaledb-secret` contains the database credentials. **We are reusing the existing TimescaleDB instance from heatpump-mqtt** for parallel operation during migration.

The secret uses the same structure as `heatpump-mqtt` with `POSTGRES_USER` and `POSTGRES_PASSWORD` keys. The connection string is constructed in the ConfigMap using environment variable substitution.

### Step 1: Get Database Credentials

First, get the credentials from the existing heatpump-mqtt secret:

```bash
# Get the database user
kubectl get secret timescaledb-secret -n heatpump-mqtt -o jsonpath='{.data.POSTGRES_USER}' | base64 -d && echo

# Get the database password
kubectl get secret timescaledb-secret -n heatpump-mqtt -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d && echo

# Get the database name
kubectl get secret timescaledb-secret -n heatpump-mqtt -o jsonpath='{.data.POSTGRES_DB}' | base64 -d && echo
```

### Step 2: Create the Secret Manifest

Create a temporary secret file `timescaledb-secret.yaml`:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: timescaledb-secret
  namespace: redpanda-sink
type: Opaque
stringData:
  POSTGRES_USER: "<USER>"
  POSTGRES_PASSWORD: "<PASSWORD>"
  POSTGRES_DB: "timescaledb"
```

Replace:
- `<USER>`: PostgreSQL username from Step 1
- `<PASSWORD>`: PostgreSQL password from Step 1
- The database name is `timescaledb` (from the existing setup)

**Note:** The connection string is automatically constructed in the ConfigMap as:
`postgresql://$(DATABASE_USER):$(DATABASE_PASSWORD)@timescaledb.heatpump-mqtt.svc.cluster.local:5432/timescaledb`

### Step 2: Seal the Secret

Use `kubeseal` to encrypt the secret:

```bash
kubeseal -f timescaledb-secret.yaml -o timescaledb-secret-sealed.yaml
```

This will create a SealedSecret that can be safely committed to Git.

### Step 3: Add to Kustomization

Add the sealed secret to `kustomization.yaml`:

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
namespace: redpanda-sink
resources:
  - namespace.yaml
  - ghcr-secret-sealed.yaml
  - timescaledb-secret-sealed.yaml  # Add this line
  - configmap.yaml
  - deployment.yaml
```

### Step 4: Apply the Secret

Apply the sealed secret to your cluster:

```bash
kubectl apply -f timescaledb-secret-sealed.yaml
```

## Reference: heatpump-mqtt Pattern

You can reference the existing `timescaledb-secret-sealed.yaml` in `gitops/apps/base/heatpump-mqtt/` as an example of the expected format.

The sealed secret should look like:

```yaml
apiVersion: bitnami.com/v1alpha1
kind: SealedSecret
metadata:
  name: timescaledb-secret
  namespace: redpanda-sink
spec:
  encryptedData:
    POSTGRES_USER: <encrypted-value>
    POSTGRES_PASSWORD: <encrypted-value>
    POSTGRES_DB: <encrypted-value>
  template:
    metadata:
      name: timescaledb-secret
      namespace: redpanda-sink
```

**Note:** This matches the structure of `heatpump-mqtt/timescaledb-secret-sealed.yaml`. You can reference that file as an example.

## Verification

After creating and applying the secret, verify it exists:

```bash
kubectl get secret timescaledb-secret -n redpanda-sink
```

The deployment will automatically use this secret via the `DATABASE_URL` environment variable.

## Troubleshooting

If the application fails to connect to the database:

1. Check the secret exists: `kubectl get secret timescaledb-secret -n redpanda-sink`
2. Verify the connection string format is correct
3. Check database network connectivity from the pod
4. Review application logs: `kubectl logs -n redpanda-sink deployment/redpanda-sink`

