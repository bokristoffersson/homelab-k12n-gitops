# homelab-k12n-gitops

GitOps repository for a home Kubernetes cluster: home automation (heat pump, smart plugs,
HomeKit), energy and temperature telemetry, and a handful of self-hosted apps. Everything
in the cluster is reconciled from this repo by FluxCD.

Documentation:
- [Architecture & Principles](docs/ARCHITECTURE.md)
- [Authentication](docs/AUTHENTICATION.md)
- [Data Flow](docs/data-flow.md) and [Kafka topics](docs/kafka-topics.md)
- [Monitoring](docs/monitoring.md)
- [Database Backup Setup](docs/BACKUP_SETUP.md)
- [Flux Setup](docs/FLUX_SETUP.md)
- [M720q migration runbook](docs/MIGRATION_M720Q.md)
- [Ansible provisioning](ansible/README.md)

## Cluster

k3s with embedded etcd, provisioned by Ansible and hardened after the k3s CIS guide
(secrets encryption, protect-kernel-defaults, audit log, Pod Security Admission
`baseline`). All nodes run Ubuntu 26.04 LTS and use static IPs (no mDNS).

| Node    | IP             | Arch  | Role                  | Notes |
|---------|----------------|-------|-----------------------|-------|
| `m720q` | 192.168.50.212 | amd64 | control-plane + etcd  | Lenovo M720q, 1 TB NVMe; primary Longhorn storage node |
| `pi0`   | 192.168.50.210 | arm64 | agent                 | Raspberry Pi 4 on SD card; compute only (no Longhorn). Homebridge pinned here for HomeKit |
| `pi1`   | 192.168.50.211 | arm64 | agent                 | Raspberry Pi 5 with NVMe; second Longhorn storage node; runs Prometheus |

- **Storage**: Longhorn on `m720q` + `pi1`, default replica count 2.
- **Multi-arch**: workloads can schedule on any node, so every image is built for both
  `linux/amd64` and `linux/arm64`.
- **Networking**: Traefik (Flux-managed, namespace `traefik`) is the only ingress
  controller; k3s's bundled Traefik is disabled. Public traffic arrives via a Cloudflare
  Tunnel (TLS terminates at Cloudflare; in-cluster traffic is HTTP). ServiceLB (klipper)
  exposes LAN services: Mosquitto `:1883`, Pi-hole DNS `:53`, Minecraft `:25565`.
- **Auth**: Authelia (OIDC, config-as-code, file user backend) at `auth.k12n.com`, with
  oauth2-proxy as a Traefik ForwardAuth gateway.

## Repository layout

```
gitops/
├── clusters/homelab/        # Flux entrypoint: infrastructure.yaml + apps.yaml (Kustomizations)
├── infrastructure/
│   ├── sources/             # HelmRepositories
│   ├── crds/                # Vendored CRDs (Traefik, cert-manager, CSI VolumeSnapshot)
│   ├── storage/             # Longhorn
│   ├── controllers/         # cert-manager, sealed-secrets, Traefik, cloudflared
│   └── actions-runner/      # Self-hosted GitHub Actions runners (ARC)
└── apps/
    ├── base/                # Per-app base manifests
    └── homelab/             # Homelab overlays (one Flux Kustomization each)
applications/                # Source code for the in-house services (Rust, React)
ansible/                     # Node provisioning and k3s hardening
docs/                        # Architecture, runbooks, ADRs (also served as Backstage TechDocs)
```

Flux reconciles `gitops/clusters/homelab` from `main`. Infrastructure comes first
(`sources`, `crds`, `storage` → `controllers`); apps declare `dependsOn` among
themselves (for example `mqtt-kafka-bridge` waits for `mosquitto` and `redpanda-v2`).

## Workloads

**Data pipeline**

```
IoT device → Mosquitto (MQTT) → mqtt-kafka-bridge → Redpanda → TimescaleDB sink → TimescaleDB
                                                                                   ↓
                                                                     homelab-api → heatpump-web
```

| Component | Description |
|---|---|
| `mosquitto` | MQTT broker for Tasmota plugs, the Shelly H&T sensor and the ThermIQ heat pump module |
| `mqtt-kafka-bridge` | Redpanda Connect: MQTT → Redpanda |
| `redpanda-v2` | Single-node Redpanda; topics managed by an `rpk` Job |
| `timescaledb` | Telemetry database, plus the Redpanda Connect sink that writes into it |

**Applications** (source in `applications/`)

| App | Description |
|---|---|
| `homelab-api` | Rust/Axum, read-only REST API over TimescaleDB |
| `heatpump-web` | React/TypeScript SPA at `homelab.k12n.com` |
| `homelab-settings` | Rust settings API + outbox processor + PostgreSQL; drives heat pump and plug commands over MQTT, with scheduling and desired-state reconcile |
| `energy-ws` | WebSocket stream of real-time energy data from Redpanda |
| `spotprice` | Electricity spot price API |
| `homelab-chat-api` | LLM-backed chat agent (SSE) with tools over the homelab APIs |
| `gasa` | Driving-practice booking app at `gasa.k12n.com` (API + web + PostgreSQL) |
| `backstage` | Developer portal and TechDocs at `homelab.k12n.com/admin/backstage` |

**Platform and home services**

| Service | Description |
|---|---|
| `authelia`, `oauth2-proxy` | Identity provider and auth gateway |
| `homebridge` | HomeKit bridge for the plugs and the heat pump (publishes straight to MQTT) |
| `pihole` | LAN DNS at `pihole.k12n.com`, with metrics exporter |
| `minecraft` | Minecraft server, pinned to `m720q` |
| `prometheus`, `loki`, `alloy`, `grafana` | kube-prometheus-stack, Loki + Alloy for logs, Grafana at `grafana.k12n.com` |

Alerting reaches Slack two ways: Alertmanager, and the `monitoring-health-check` CronJob
(data freshness, consumer lag, failed device commands). A healthchecks.io dead man's switch
covers both.

## CI/CD

- Each service in `applications/` has a GitHub Actions workflow that runs tests and then
  builds a multi-arch image (native `ubuntu-24.04` and `ubuntu-24.04-arm` runners) and
  pushes it to GHCR.
- Every PR gets an AI architectural review against [CLAUDE.md](CLAUDE.md)
  (`.github/workflows/arch-review.yml`, see [docs/ARCH_REVIEW.md](docs/ARCH_REVIEW.md)).
- Work goes on feature branches with PRs. Flux deploys what lands on `main`.
- Kubernetes does not pick up a new image or ConfigMap on its own, so a deployment needs
  `kubectl rollout restart` after its image is rebuilt.

## Secrets

Secrets are committed only as Bitnami SealedSecrets (`*-sealed.yaml`). `kubeseal`
fetches the controller certificate through the Kubernetes API, so sealing works from any
machine with cluster access:

```bash
kubectl create secret generic <name> -n <namespace> \
  --from-literal=KEY=value --dry-run=client -o yaml | \
kubeseal --namespace <namespace> \
  --controller-name sealed-secrets --controller-namespace kube-system \
  --format=yaml > <name>-sealed.yaml
```

## Database migrations

Migrations are SQL files run by Kubernetes Jobs. Add a file under the app's
`migrations/` directory and list it in that app's `kustomization.yaml`
`configMapGenerator`. Never run them by hand with `kubectl exec`. See
[CLAUDE.md](CLAUDE.md) for the full workflow.

## Backups

| What | How |
|---|---|
| TimescaleDB | `pg_dump` CronJob to S3, daily 02:00 |
| homelab-settings PostgreSQL | CronJob to S3, daily 02:15 |
| gasa PostgreSQL | CronJob to S3, daily 02:45 |
| Backstage PostgreSQL | CronJob to S3, daily 02:00 |
| etcd | k3s scheduled snapshots, stored locally on `m720q` (S3 upload is supported by the Ansible template but not enabled) |

## Rebuilding from scratch

1. Provision the nodes with Ansible: `ansible-playbook site.yml` (see
   [ansible/README.md](ansible/README.md)).
2. Install the Flux Operator and create the GitHub App secret, then apply
   [flux.yaml](flux.yaml). It syncs `gitops/clusters/homelab` from `main`
   ([docs/FLUX_SETUP.md](docs/FLUX_SETUP.md)).
3. Sealed secrets only decrypt with the original controller key, so restore it or reseal
   every secret.
4. [docs/MIGRATION_M720Q.md](docs/MIGRATION_M720Q.md) records the pitfalls hit during
   the last rebuild.

## Common commands

```bash
flux get kustomizations                                   # sync status
flux reconcile kustomization <name> --with-source         # pull and apply now
kubectl rollout restart deployment/<app> -n <namespace>   # pick up a new image/ConfigMap
make help                                                 # port-forwards, Redpanda/MQTT helpers
```
