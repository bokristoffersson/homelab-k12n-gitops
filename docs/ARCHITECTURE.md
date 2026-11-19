# Architecture & Principles

How this GitOps repository is structured, and why.

## Git is the source of truth

Everything running in the cluster is declared in this repository. FluxCD reads `main`
and makes the cluster match it.

```
commit on main → Flux pulls the repo → Flux applies the manifests → cluster converges
```

Nothing is applied by hand. A manual `kubectl edit` is overwritten on the next
reconcile, and that is intended.

## Repository structure

```
gitops/
├── clusters/homelab/     # What Flux reads first: infrastructure.yaml and apps.yaml
├── infrastructure/       # The platform: sources, CRDs, storage, controllers
└── apps/<app>/           # One directory per workload
```

| Layer | Path | Contains |
|---|---|---|
| Cluster | `gitops/clusters/homelab/` | One Flux `Kustomization` per app and per infrastructure group, with the `dependsOn` ordering between them |
| Infrastructure | `gitops/infrastructure/` | Helm repositories, vendored CRDs, Longhorn, cert-manager, sealed-secrets, Traefik, the Cloudflare tunnel, etcd snapshot config |
| Apps | `gitops/apps/<app>/` | The manifests for one workload, usually in its own namespace |

Dependencies point one way: apps depend on infrastructure, never the reverse.
Infrastructure is applied in the order `sources`, `crds`, `storage`, then `controllers`.
Apps declare what they need, for example `mqtt-kafka-bridge` waits for `mosquitto` and
`redpanda-v2`.

### One cluster, no overlays

There is a single cluster, so each app directory is plain kustomize and is applied as it
is. The repository used to have a `base/` and `homelab/` overlay split; 22 of the 24
overlays did nothing but re-export their base, so it was removed. If a second
environment is ever needed, add overlays then.

New features are developed and tested locally in the application's own source tree
(`applications/<app>/`) and then deployed to the homelab.

## Design decisions

**Kustomize for our own apps, Helm for third-party charts.** Plain manifests show
exactly what gets applied. Charts are used where upstream maintains them: Prometheus,
Grafana, Loki, Alloy, Redpanda, Pi-hole, Traefik, Longhorn, cert-manager,
sealed-secrets.

**A namespace per app.** It gives each workload its own RBAC and resource boundary and
lets Flux prune one app without touching another.

**Secrets as SealedSecrets.** Only the cluster's controller can decrypt them, so the
encrypted form is safe to commit. Plain `Secret` manifests are never committed.

**Database migrations as Jobs.** Schema changes are SQL files in the app's directory,
applied by a Kubernetes Job that Flux manages. See [CLAUDE.md](../CLAUDE.md) for the
workflow.

**Checks before merge.** Every PR is rendered and schema-validated
(`scripts/validate-gitops.sh`), linted, and reviewed by an AI agent. See the CI/CD
section of the [README](../README.md).

## Terms

| Term | Meaning |
|---|---|
| Reconciliation | Flux comparing Git with the cluster and applying the difference |
| Kustomization (Flux) | A Flux resource that applies one directory of manifests |
| kustomization.yaml | The kustomize file inside a directory that lists its resources |
| HelmRelease | A Flux resource that installs and upgrades a Helm chart |
| Source | A Git or Helm repository that Flux watches |
| SealedSecret | An encrypted secret that is safe to commit |

## What happens when something fails

| Situation | Result |
|---|---|
| GitHub is unreachable | The cluster keeps running on its current state. Flux retries; no new changes arrive until it recovers. |
| A Flux controller crashes | Workloads are unaffected. The controller restarts and reconciliation resumes. |
| A broken manifest is merged | The Flux Kustomization for that app goes `Ready=False` and the previous objects stay in place. Revert the commit. |
| Someone edits the cluster by hand | The change is overwritten on the next reconcile. |

Check the state with:

```bash
flux get kustomizations
flux get helmreleases -A
```

## Changing things

**Add an app**

1. Create `gitops/apps/myapp/` with its manifests and a `kustomization.yaml`.
2. Add a Flux `Kustomization` for it in `gitops/clusters/homelab/apps.yaml`, with
   `dependsOn` for anything it needs.
3. Run `make lint`, open a PR, merge.

**Add infrastructure**

1. Add a directory under `gitops/infrastructure/controllers/` and list it in that
   directory's `kustomization.yaml`.
2. If it comes from a Helm chart, add its `HelmRepository` under
   `gitops/infrastructure/sources/`.
3. Apps that need it declare `dependsOn: infrastructure-controllers`.

**Roll back**

Revert the commit on a branch and merge it. Flux applies the previous state.

## Observability

- **Metrics**: Prometheus (kube-prometheus-stack), including Flux's own metrics.
- **Logs**: Alloy ships pod logs to Loki.
- **Dashboards**: Grafana at `grafana.k12n.com`.
- **Alerts**: Alertmanager to Slack, plus a health-check CronJob for data freshness.

See [monitoring.md](monitoring.md).

## References

- [Flux concepts](https://fluxcd.io/flux/concepts/)
- [OpenGitOps principles](https://opengitops.dev/)
