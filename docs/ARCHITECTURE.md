# Architecture & Principles

A short guide to how this GitOps setup is structured and why.

## Core Principle: Git as Single Source of Truth

Everything in the cluster is declared in Git. No manual `kubectl apply`. No surprises.

```
Git Commit → Flux Detects Change → Cluster Converges to Desired State
```

This is **declarative infrastructure**: we describe the *what*, not the *how*.

## Bounded Contexts (DDD)

The repository is organized into clear domains, each with distinct responsibilities:

### 1. Infrastructure Domain (`gitops/infrastructure/`)

**Responsibility**: Platform capabilities - the foundation everything else runs on.

**Aggregates**:
- **Controllers**: Flux, cert-manager, sealed-secrets, Traefik
- **Sources**: Helm repositories, container registries
- **Policies**: Network policies, RBAC, resource quotas

**Invariants**:
- Infrastructure must be ready before applications
- Controllers are cluster-scoped (no namespace ownership)

### 2. Application Domain (`gitops/apps/`)

**Responsibility**: Business workloads - the actual services that do useful work.

**Aggregates**:
- **Redpanda**: Message streaming platform
- **Monitoring**: Prometheus, Grafana, Alloy, Loki
- **Custom Apps**: heatpump-mqtt, mqtt-input, redpanda-sink

**Invariants**:
- Apps depend on infrastructure being healthy
- Each app owns its namespace
- Apps can't modify infrastructure

### 3. Cluster Domain (`gitops/clusters/`)

**Responsibility**: Environment-specific configuration and orchestration.

**Aggregates**:
- **Homelab**: Production configuration
- **Local**: Development configuration

**Invariants**:
- One cluster = one set of Kustomizations
- Cluster configs reference base + overlays
- No cross-cluster dependencies

## Domain Events & Reconciliation

In DDD, domain events trigger behavior. In GitOps, **git commits are domain events**:

```
Developer commits → GitHub webhook → Flux polls repository →
Reconciliation loop → Cluster state converges → Event: Reconciled
```

**Reconciliation** is the core domain service:
- Reads desired state (Git)
- Compares with actual state (Cluster)
- Takes corrective action (Apply changes)
- Repeats every N minutes

This is **eventual consistency** by design.

## Layered Architecture

```
┌─────────────────────────────────────┐
│   Clusters (Environment Configs)   │  ← Orchestration Layer
├─────────────────────────────────────┤
│         Apps (Overlays)             │  ← Business Logic Layer
├─────────────────────────────────────┤
│      Apps (Base Manifests)          │  ← Domain Model Layer
├─────────────────────────────────────┤
│  Infrastructure (Controllers)       │  ← Platform Layer
├─────────────────────────────────────┤
│      Kubernetes Cluster (K3s)       │  ← Infrastructure Layer
└─────────────────────────────────────┘
```

**Dependency Rule**: Outer layers depend on inner layers, never the reverse.

- Clusters can reference Apps and Infrastructure
- Apps can reference Infrastructure (via dependencies)
- Infrastructure is self-contained
- Base manifests have zero dependencies

## Ubiquitous Language

Terms we use consistently across the codebase:

| Term | Meaning |
|------|---------|
| **Reconciliation** | Process of making cluster state match Git |
| **Kustomization** | Flux resource that applies a directory of manifests |
| **HelmRelease** | Flux resource that manages a Helm chart |
| **Source** | Git repo or Helm repo that Flux watches |
| **Overlay** | Environment-specific patches to base configs |
| **Sealed Secret** | Encrypted secret safe to commit to Git |
| **Bootstrap** | Initial Flux installation connecting to Git |

## Strategic Design Decisions

### Why GitOps?

- **Audit Trail**: Every change is a git commit
- **Rollback**: `git revert` is instant disaster recovery
- **Review**: Pull requests before production changes
- **Declarative**: Describe desired state, not imperative steps

### Why Kustomize over Helm for Apps?

- **Transparency**: See exactly what gets applied
- **Simplicity**: Base + patches, no templating magic
- **Flexibility**: Mix Helm charts with raw manifests

We use Helm for third-party charts (Prometheus, Redpanda), Kustomize for our apps.

### Why Separate Local from Production?

**Local** (Development):
- Plain Kubernetes secrets (never committed)
- Reduced resources
- Direct `kubectl apply -k`
- Fast iteration

**Production**:
- Sealed secrets (safe to commit)
- Full resources
- Flux reconciliation
- Audit trail

This is **context mapping** in DDD - two contexts with different rules.

### Why Namespace-per-App?

Each application is a **bounded context**. Namespaces provide:
- Resource isolation
- RBAC boundaries
- Clear ownership
- Independent lifecycle

## Data Flow

```
┌──────────┐     ┌──────────┐     ┌────────────┐     ┌─────────┐
│Developer │────>│   Git    │────>│    Flux    │────>│Cluster  │
│  (Edit)  │     │ (Source) │     │(Controller)│     │ (State) │
└──────────┘     └──────────┘     └────────────┘     └─────────┘
                                         │
                                         ↓
                                   ┌────────────┐
                                   │  Metrics   │
                                   │  Logs      │
                                   │  Alerts    │
                                   └────────────┘
```

1. Developer edits manifests locally
2. Commits to Git (GitHub)
3. Flux polls repository (1min interval)
4. Flux applies changes to cluster
5. Cluster converges to desired state
6. Observability captures metrics/logs

## Anti-Patterns We Avoid

❌ **Manual Changes**: `kubectl edit` creates drift from Git
✅ **Edit Git, let Flux sync**

❌ **Imperative Scripts**: `kubectl apply -f` in CI/CD
✅ **Declarative manifests in Git**

❌ **Secrets in Git**: Plain secrets committed
✅ **Sealed secrets encrypted**

❌ **Monolithic Configs**: One giant YAML
✅ **Modular, composable manifests**

❌ **Environment Duplication**: Copy/paste configs
✅ **Base + overlays pattern**

## Failure Modes & Recovery

**What happens if...**

**Git is down?**
- Cluster keeps running (Git is source of truth, not runtime dependency)
- Flux retries on interval
- No new changes until Git recovers

**Flux crashes?**
- Cluster keeps running
- Flux restarts automatically (Deployment)
- Reconciliation resumes

**Bad manifest committed?**
- Flux applies it (eventual consistency)
- Errors appear in Flux logs
- `git revert` and push to fix
- Flux reconciles to last good state

**Manual change in cluster?**
- Flux detects drift on next reconcile
- Overwrites manual change with Git state
- This is a feature, not a bug

## Testing Strategy

**Local Development**:
```bash
kubectl apply -k gitops/apps/local/redpanda-v2
# Instant feedback, no Git needed
```

**Production Deployment**:
```bash
git commit -m "feat: update redpanda"
git push
# Flux reconciles in ~1 minute
```

**Validation**:
```bash
flux get kustomizations  # Check health
flux logs --follow       # Watch reconciliation
```

## Observability

**Three Pillars**:

1. **Metrics**: Prometheus scrapes all services
2. **Logs**: Alloy ships to Loki
3. **Traces**: (Future) OpenTelemetry

**GitOps-specific**:
- Flux exports Prometheus metrics
- Reconciliation status in Flux CRDs
- Git commit SHA in annotations

## Evolution & Extensions

**Adding a New App**:
1. Create `gitops/apps/base/myapp/`
2. Create `gitops/apps/homelab/myapp/kustomization.yaml`
3. Add to `gitops/clusters/homelab/apps.yaml`
4. Commit, push, done

**Adding a New Environment**:
1. Create `gitops/clusters/staging/`
2. Create environment-specific overlays
3. Bootstrap Flux pointing to new path
4. Independent lifecycle from production

**Adding Infrastructure**:
1. Add to `gitops/infrastructure/controllers/`
2. Ensure apps have `dependsOn: infrastructure-controllers`
3. Commit, push, Flux handles ordering

## Key Metrics

**DORA Metrics** (DevOps Research & Assessment):

- **Deployment Frequency**: Every git push (continuous)
- **Lead Time**: ~1 minute (Git commit → deployed)
- **MTTR**: ~2 minutes (git revert → fixed)
- **Change Failure Rate**: Low (Git review + Flux validation)

## References

- [Flux Architecture](https://fluxcd.io/flux/concepts/)
- [Domain-Driven Design](https://martinfowler.com/bliki/DomainDrivenDesign.html)
- [GitOps Principles](https://opengitops.dev/)
- [Kustomize Overlay Pattern](https://kubectl.docs.kubernetes.io/references/kustomize/)

---

**TL;DR**: Git is truth. Flux reconciles. Apps are domains. Environments are contexts. Everything is declarative. Recovery is `git revert`.
