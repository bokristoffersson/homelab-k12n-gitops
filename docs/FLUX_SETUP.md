# Flux Setup

How Flux is installed on the cluster, and how to bootstrap it again on an empty one.

Flux is managed by the [Flux Operator](https://fluxcd.control-plane.io/operator/). The
operator reads one `FluxInstance` resource, [`flux.yaml`](../flux.yaml) in the repository
root, and from it installs the Flux controllers and points them at this repository.

```
flux-operator (Helm)  →  FluxInstance (flux.yaml)  →  Flux controllers
                                                         │
                              syncs gitops/clusters/homelab from main
```

`flux.yaml` sets:

- **Sync source**: this repository, `refs/heads/main`, path `gitops/clusters/homelab`.
- **Auth**: `provider: github` with the pull secret `flux-system`, which holds GitHub
  App credentials.
- **Controllers**: source, kustomize, helm, notification, image-reflector and
  image-automation.
- **Version**: `2.x`, so the operator follows Flux minor releases.

## Bootstrapping an empty cluster

Prerequisites: `kubectl` pointed at the cluster, plus the `flux` and `helm` CLIs.

### 1. Restore the sealed-secrets key

Do this before Flux starts. SealedSecrets in this repository only decrypt with the
original controller key. Without it, every secret has to be sealed again.

```bash
kubectl apply -f sealed-secrets-keys-backup.yaml
```

### 2. Install the Flux Operator

```bash
helm install flux-operator \
  oci://ghcr.io/controlplaneio-fluxcd/charts/flux-operator \
  --namespace flux-system --create-namespace
```

The operator is the one component that is not managed from Git.

### 3. Create the GitHub App secret

Flux reads the repository through a GitHub App with read access to **Contents** and
**Metadata**, installed on this repository. Generate a private key for the app, then:

```bash
flux create secret githubapp flux-system \
  --app-id=<app id> \
  --app-installation-id=<installation id> \
  --app-private-key=./private-key.pem
```

Delete the `.pem` file afterwards.

### 4. Apply the FluxInstance

```bash
kubectl apply -f flux.yaml
```

### 5. Watch it converge

```bash
flux get kustomizations
flux get helmreleases -A
kubectl get sealedsecrets -A
```

Infrastructure is applied first (`sources`, `crds`, `storage`, then `controllers`),
followed by the apps in dependency order. The first run takes several minutes.

[MIGRATION_M720Q.md](MIGRATION_M720Q.md) records the problems hit during the last
rebuild, including a CRD ordering issue on an empty cluster.

## Day-to-day commands

```bash
flux get kustomizations                              # status of every app
flux reconcile kustomization <name> --with-source    # pull and apply now
flux suspend kustomization <name>                    # pause during maintenance
flux resume kustomization <name>
```

## Troubleshooting

**Nothing is syncing**

```bash
flux get sources git
flux logs --level=error
```

An authentication error here usually means the GitHub App key was rotated; recreate the
`flux-system` secret (step 3).

**One Kustomization is not Ready**

```bash
flux get kustomizations
kubectl describe kustomization <name> -n flux-system
```

The message names the object that failed. Run `make lint` locally to reproduce
rendering and schema errors before pushing a fix.

**A HelmRelease is stuck**

```bash
flux get helmreleases -A
kubectl describe helmrelease <name> -n <namespace>
flux reconcile helmrelease <name> -n <namespace> --force
```

## References

- [Flux documentation](https://fluxcd.io/flux/)
- [Flux Operator](https://fluxcd.control-plane.io/operator/)
- [Sealed Secrets](https://github.com/bitnami-labs/sealed-secrets)
