# Migrering: k3s till Lenovo M720q (härdat kluster)

Runbook för att migrera homelab-klustret från två Raspberry Pis (p0, p1) till en
Lenovo M720q som control-plane, med Pi:erna som agenter efteråt.

**Så används dokumentet:** mata Claude med det i början av varje arbetssession
("läs docs/MIGRATION_M720Q.md och fortsätt där vi är"). Varje steg är märkt
**[Claude]** (Claude utför) eller **[Bo]** (Claude instruerar, Bo utför —
fysiska moment eller kommandon på macOS-hosten). Bocka av steg allteftersom
(ändra `[ ]` till `[x]`) så blir dokumentet också lägeslogg.

## Beslut (fattade 2026-07-02)

| Fråga | Beslut |
|---|---|
| OS | Ubuntu Server 24.04 LTS (alla noder) |
| Migreringsstrategi | Nytt kluster + Flux-bootstrap mot samma repo; data återställs från S3-dumpar |
| Datastore | Inbäddad etcd (`cluster-init: true`), snapshots till S3 |
| Arkitektur | Multi-arch-images (amd64 + arm64) så workloads kan köra på M720q |
| Provisionering | Ansible i `ansible/` i detta repo, baserat på k3s-io/k3s-ansible |
| Härdning | k3s CIS hardening guide (protect-kernel-defaults, secrets-encryption, audit-logg, PSA) |

## Hårdvara

- **M720q**: amd64, 32 GB RAM, SATA-SSD (OS) + 1 TB NVMe (etcd/containerd + Longhorn)
- **p0, p1**: Raspberry Pi, arm64 — kör gamla klustret tills fas 5, ominstalleras sedan som agenter

## Miljönoteringar för Claude (körs i claude-box / Apple container)

- **mDNS fungerar inte i containern** — `.local`-namn resolvar inte. Använd
  alltid IP-adresser i ansible-inventory, kubeconfig och SSH-config.
- **Verktyg i boxen**: kubectl, flux, helm, kubeseal (v0.32.2), ansible +
  ansible-lint, gh, git, ssh, sshpass, yq, jq, rg/fd, dig/ping, rust, uv.
  Saknas något: be Bo lägga till det i `~/Development/apple-container/Containerfile`
  och bygga om.
- **SSH**: nyckeln `~/.ssh/id_p1` i den persistenta ssh-volymen återanvänds för
  alla noder. Claude får själv uppdatera `~/.ssh/config` och `known_hosts`
  (volymen persisterar mellan sessioner). Publika nyckeln måste in i
  `authorized_keys` på nya maskiner — antingen via `sshpass` + `ssh-copy-id`
  första gången, eller av Bo.
- **ansible-galaxy-collections** försvinner när containern stängs (`--rm`).
  Sätt `collections_path = ./collections` i `ansible/ansible.cfg` (katalogen
  ligger i repot/workspace och överlever), lägg `collections/` i `.gitignore`.
- **kubeseal**: kör `kubeseal --fetch-cert` mot aktiv kubectl-context — SSH
  till p1 behövs inte längre. (CLAUDE.md:s p1-instruktion gäller gamla
  klustret; uppdatera CLAUDE.md efter fas 5.)
- **Kubeconfig-byte** (t.ex. vid cutover) görs av Bo på hosten:
  `./claude-box.sh kube <context>`.
- **Versioner**: slå upp aktuell stabil k3s-version (och andra versioner) vid
  körningstillfället — hårdkoda inte från minnet.

## Nulägesfakta (repo-kartläggning 2026-07-02)

- **Flux**: flux-operator + FluxInstance (rot-`flux.yaml`), sync-path
  `gitops/clusters/homelab`, repo `github.com/bokristoffersson/homelab-k12n-gitops`.
- **Sealed secrets**: 35 st, controller (chart 2.17.7 = v0.32.2) i kube-system.
  Nyckeln MÅSTE flyttas till nya klustret, annars dekrypterar inget.
- **arm64-only images** (får inte glömmas — schemaläggs inte på amd64 förrän
  fixade): backstage, energy-ws, heatpump-web, heatpump-web-leptos,
  homelab-api, homelab-settings-api, homelab-settings-outbox-processor,
  spotprice-api. Dessutom cloudflared pinnad till `latest-arm64`
  (`gitops/infrastructure/controllers/cloudflare-tunnel/deployment.yaml`).
- **Nodbindningar**: homebridge nodeSelector `p0`
  (`gitops/apps/base/homebridge/deployment.yaml`), Prometheus nodeSelector `p1`
  (`gitops/apps/base/monitoring/helmreleases.yaml`), Longhorn-nod p1 med
  NVMe-disk (`gitops/infrastructure/storage/longhorn/node-nvme.yaml`).
- **Backup**: ENDAST pg_dump→S3-cronjobs (timescaledb, homelab-settings,
  backstage). Longhorn har inget S3 backup target. O-backat state: Homebridge
  (HomeKit-parning), Authelia local storage (TOTP-registreringar), Pi-hole,
  Grafana, Mosquitto, Redpanda-logg, Loki.
- **actions-runner** kör on-cluster med sealad kubeconfig — innehållet pekar på
  gamla klustret och måste re-sealas mot nya.

---

## Fas 0 — Förberedelser (gamla klustret kör, noll risk)

- [x] **[Bo]** Exportera sealed-secrets-nyckeln på hosten (context `homelab`,
  gamla klustret) och spara i lösenordshanteraren — ALDRIG i git:
  ```bash
  kubectl get secret -n kube-system \
    -l sealedsecrets.bitnami.com/sealed-secrets-key \
    -o yaml > ~/sealed-secrets-keys-backup.yaml
  ```
- [x] **[Claude]** PR: multi-arch-byggen. Lägg till amd64 i de 8 workflows
  (lista ovan). Rust: matrix-bygg nativt (`ubuntu-24.04` för amd64,
  `ubuntu-24.04-arm` för arm64) + manifest-merge-steg — bygg INTE Rust under
  QEMU. Verifiera efteråt med `gh run watch` + kontrollera plattformar i GHCR.
  → PR #127 mergad. Alla 8 images bekräftat amd64+arm64 i GHCR (registry API,
  `:main`-taggen). En app (heatpump-web-leptos) failade första push-till-main
  körningen — `wasm-bindgen-test` → `wit-bindgen 0.51.0` kräver `edition2024`,
  som `rust:1.83` i Dockerfilen inte klarade av. Fixat i PR #128 (bump till
  `rust:1.96`), mergad och grön.
- [x] **[Claude]** Samma PR eller egen: byt cloudflared till multi-arch-tagg
  med pinnad version (inte `latest`). → `2026.6.1` (bekräftat amd64+arm64 på Docker Hub).
- [x] **[Claude]** PR: städa nodbindningar — ta bort homebridge-nodeSelector
  `p0`, Prometheus-nodeSelector `p1` och `longhorn/node-nvme.yaml` (på nya
  noden monteras NVMe direkt på Longhorns default-path `/var/lib/longhorn`).
  → PR #130 mergad. **OBS:** Prometheus-nodeSelectorn fick återinföras i PR
  #133 — gamla klustret kör fortfarande på riktigt tills fas 5, och utan
  pinning hamnade Prometheus direkt på p0 (96% CPU), samma mönster som
  tidigare destabiliserat redpandas raft. Ta bort den nodeSelectorn igen
  först vid den faktiska fas 5-cutovern, inte som fas 0-städning.
- [x] **[Claude]** Samma PR: sätt Longhorns `defaultClassReplicaCount: 1` i
  HelmRelease-values (en nod kan inte hålla 3 repliker — volymer fastnar
  annars i degraded). Höjs i fas 5. → Mergad i PR #130. Påverkar bara nya
  volymer, inga befintliga volymer på gamla klustret krympte.
- [x] **[Claude]** Verifiera färska S3-dumpar: kolla senaste lyckade körning av
  backup-cronjobs för timescaledb, homelab-settings, backstage; testläs en dump.
  → Hittade att timescaledb- och homelab-settings-backuperna varit trasiga i
  minst 3 dagar (hårdkodat fel lösenord, PR #131) och att backstage-backupen
  aldrig backat upp riktig data (dumpade fel databas — Backstage har en
  databas per plugin, PR #132: bytt till `pg_dumpall`). Alla tre verifierade
  med riktig `pg_dump` mot klustret efter fix — fungerar nu.
- [x] **[Bo + Claude]** Rädda o-backat state (välj ambitionsnivå): tar/`kubectl cp`
  ut Homebridge-volymen (HomeKit-parningen!), Authelia-storage (TOTP),
  Pi-hole-config, ev. Grafana. Acceptera förlust av Loki-loggar och
  Redpanda-logg (topics återskapas av rpk-jobbet). → Klart 2026-07-03: tar +
  `kubectl cp` + uppladdat till `s3://k12n-homelab-db-backups/pre-migration-state/20260703/`
  (samma bucket som pg-dumparna, egen prefix):
  - `homebridge-state-backup.tar.gz` (23M, hela `/homebridge` utom loggfil —
    persist/ har HomeKit-parningen)
  - `authelia-data-backup.tar.gz` (275K, `/data` — TOTP-registreringar)
  - `pihole-config-backup.tar.gz` (3.1M, `/etc/pihole` utom `pihole-FTL.db*`
    som bara är query-statistik, ~1GB, motsvarande "logg" - inte config)
  - `grafana-data-backup.tar.gz` (154K, `/var/lib/grafana` — grafana.db med
    ev. manuellt skapade dashboards/users utöver GitOps-provisionerade)
  Detta är en engångssnapshot, inte en löpande backup — färskare data
  (nya HomeKit-parningar, TOTP-registreringar) före fas 5-cutover fångas
  inte automatiskt.
- [x] **[Bo]** Kartlägg IP-beroenden: vilken broker-IP pekar Shelly-sensorn på?
  Pekar några klienter på Pi-hole som DNS? Bestäm ny IP-plan (DHCP-reservation
  för M720q; peka om devices vid cutover i fas 5). → Shelly-sensorn kör DHCP,
  ingen hårdkodad DNS/broker-IP hittad. DHCP-reservationer 192.168.50.210/.211
  inlagda i routern; DNS manuellt satt på de klienter som inte respekterar
  DHCP-DNS (vanligt för vissa smart-TV/IoT/enheter med private DNS).
- [x] **[Bo]** Bygg om claude-boxen så nya verktygen finns:
  ```bash
  cd ~/Development/apple-container
  container build --tag claude-box:latest --file Containerfile .
  ```

## Fas 1 — M720q: fysisk setup + OS  *(Claude instruerar, Bo utför)*

- [ ] **[Bo]** Skapa USB-sticka med **Ubuntu Server 24.04 LTS (amd64)** —
  ladda ner ISO, skriv med balenaEtcher eller
  `sudo dd if=ubuntu-24.04-live-server-amd64.iso of=/dev/diskN bs=4m`.
- [ ] **[Bo]** BIOS på M720q (F1 vid boot):
  - **Power → After Power Loss: Power On** (homelab-krav: startar själv efter strömavbrott)
  - Boot order: USB först (tillfälligt)
  - Intel VT-x/VT-d: enabled
  - Secure Boot: kan vara på (Ubuntu stödjer det)
- [ ] **[Bo]** Installera Ubuntu på **SATA-SSD:n** (välj rätt disk — INTE NVMe:n!):
  - Hostname: `m720q`, användare: `bo`
  - "Install OpenSSH server": JA; importera gärna SSH-nyckel från GitHub
  - Ingen extra snap-paketering behövs
- [ ] **[Bo]** DHCP-reservation för M720q i routern; notera IP:t här: `______`
- [ ] **[Bo]** Lägg in claude-boxens pubnyckel så Ansible når maskinen:
  ```bash
  cd ~/Development/apple-container
  ./claude-box.sh ssh-setup pubkey | ssh bo@<m720q-ip> 'cat >> ~/.ssh/authorized_keys'
  ```
- [ ] **[Claude]** Lägg till `m720q` i `~/.ssh/config` i boxen (IP från ovan) och
  verifiera `ssh m720q 'hostname && lsblk'` — kontrollera att NVMe:n syns.

## Fas 2 — Ansible-struktur

- [ ] **[Claude]** Skapa `ansible/` i detta repo (egen feature-branch/PR):
  ```
  ansible/
  ├── ansible.cfg            # collections_path = ./collections
  ├── inventory.yml          # m720q i [server]; p0/p1 läggs i [agent] i fas 5 — IP:n, inte .local
  ├── requirements.yml       # galaxy-collections
  ├── site.yml
  └── roles/
      ├── common/            # OS-härdning, alla noder
      ├── k3s-server/
      └── k3s-agent/
  ```
- [ ] **[Claude]** `common`-rollen:
  - ufw: allow 22/tcp (LAN), 6443/tcp (LAN), 10250/tcp + 8472/udp (endast klusternoder),
    servicelb-portar för Mosquitto m.m. från LAN; default deny incoming
  - SSH-härdning: `PasswordAuthentication no`, `PermitRootLogin no`
  - unattended-upgrades
  - `/etc/sysctl.d/90-kubelet.conf` (krävs av `protect-kernel-defaults`):
    `vm.panic_on_oom=0`, `kernel.panic=10`, `kernel.panic_on_oops=1`
- [ ] **[Claude]** Disklayout-tasks (eller engångskörning) för NVMe via LVM:
  - `~200G` → `/var/lib/rancher` (etcd + containerd på NVMe)
  - resten (~730G) → `/var/lib/longhorn`
  - fstab-entries, ext4
- [ ] **[Claude]** `k3s-server`-rollen: k3s-binär (pinnad version — slå upp aktuell
  stabil), `/etc/rancher/k3s/config.yaml` + audit-policy + PSA-config
  (se fas 3), systemd-enhet, token-hantering.
- [ ] **[Claude]** Kör `ansible-lint` rent innan PR.

## Fas 3 — Härdad k3s-server på M720q

`/etc/rancher/k3s/config.yaml` enligt k3s CIS hardening guide:

```yaml
cluster-init: true              # inbäddad etcd
secrets-encryption: true
protect-kernel-defaults: true
tls-san:
  - <m720q-ip>
kube-apiserver-arg:
  - admission-control-config-file=/var/lib/rancher/k3s/server/psa.yaml
  - audit-log-path=/var/lib/rancher/k3s/server/logs/audit.log
  - audit-policy-file=/var/lib/rancher/k3s/server/audit-policy.yaml
  - audit-log-maxage=30
  - audit-log-maxbackup=10
  - audit-log-maxsize=100
kube-controller-manager-arg:
  - terminated-pod-gc-threshold=10
etcd-snapshot-schedule-cron: "0 3 * * *"
etcd-snapshot-retention: 14
# + etcd-s3: true med bucket/region/creds (samma S3 som pg-backuperna)
```

PSA-config: `enforce: baseline` globalt, `audit/warn: restricted`, undantag
(exemptions/namespace-labels) för `kube-system` och `longhorn-system`
(Longhorn kräver privileged). **Ingen NoSchedule-taint** på control-plane —
M720q ska köra alla workloads tills Pi:erna är med.

- [ ] **[Claude]** Kör playbooken mot m720q.
- [ ] **[Claude]** Verifiera: `ssh m720q 'sudo k3s kubectl get nodes'`,
  ta manuell etcd-snapshot (`k3s etcd-snapshot save`) och bekräfta S3-uppladdning.
- [ ] **[Bo]** Hämta kubeconfig till hosten och gör den till context, t.ex.:
  ```bash
  ssh bo@<m720q-ip> 'sudo cat /etc/rancher/k3s/k3s.yaml' \
    | sed 's/127.0.0.1/<m720q-ip>/' > ~/.kube/m720q.yaml
  # merga in i ~/.kube/config som context "homelab-new"
  ```
  Kör sedan `./claude-box.sh kube homelab-new` så Claude når nya klustret.

## Fas 4 — Flux-bootstrap + dataåterställning

Ordningen är viktig — sealed-secrets-nyckeln FÖRE Flux:

- [ ] **[Bo]** Återställ nyckeln (host, context `homelab-new`):
  `kubectl apply -f ~/sealed-secrets-keys-backup.yaml`
- [ ] **[Claude]** Installera flux-operator (Helm) + git-auth-secret för repot,
  applicera rot-`flux.yaml` (FluxInstance).
- [ ] **[Claude]** Följ utrullningen (`flux get kustomizations`), bekräfta att
  sealed secrets dekrypteras: `kubectl get sealedsecrets -A` utan fel i status.
- [ ] **[Claude]** Återställ databaser från S3-dumpar (engångs-restore via
  `kubectl exec psql < dump` är OK — det är migrations, inte restores, som är
  GitOps): timescaledb, homelab-settings, backstage. Verifiera radantal mot
  förväntan.
- [ ] **[Claude]** Återställ räddat state från fas 0 (Homebridge, Authelia, Pi-hole).
- [ ] **[Claude]** Re-seala actions-runnerns kubeconfig — gamla innehållet pekar
  på p1.local:6443. Generera ny mot m720q, seala med `kubeseal --fetch-cert`
  mot nya klustret, committa.
- [ ] **[Claude]** Smoke-test via port-forward: homelab-api, heatpump-web,
  Grafana, Authelia-login.

## Fas 5 — Cutover + Pi:erna som agenter

- [ ] **[Claude]** Skala ner cloudflared på GAMLA klustret först (annars
  round-robinar tunneln mellan klustren), verifiera sedan att
  `https://homelab.k12n.com` och `https://auth.k12n.com` svarar från nya.
- [ ] **[Bo]** Peka om Shelly-sensorns MQTT-broker till ny IP; peka om
  Pi-hole-DNS-klienter enligt IP-planen från fas 0.
- [ ] **[Claude]** Övervaka dataflödet ~1 dygn: Shelly → Mosquitto → Redpanda →
  TimescaleDB → homelab-api; kontrollera att grafer fylls på och att
  redpanda-sink/settings-consumern är friska.
- [ ] **[Bo]** Ominstallera Pi:erna EN i taget med Ubuntu Server 24.04 (arm64):
  flasha SD/SSD, hostname p0/p1, OpenSSH på, lägg in claude-box-pubnyckeln
  (samma kommando som fas 1).
- [ ] **[Claude]** Lägg p0/p1 i `[agent]` i inventoryt, kör playbooken
  (agent-config: `protect-kernel-defaults: true` + sysctls + server-URL + token).
  Verifiera `kubectl get nodes` = 3 Ready.
- [ ] **[Claude]** PR: höj Longhorns default replica count till 2, verifiera att
  volymer replikerar över noderna.
- [ ] **[Bo]** Riv gamla klustret först när allt ovan varit grönt några dagar.
- [ ] **[Claude]** PR: uppdatera CLAUDE.md (kubeseal-flödet via boxen i stället
  för p1, nodnamn, ev. IP-referenser).

## Fas 6 — Efter-härdning (egna småprojekt, efter migreringen)

- [ ] Kör CIS self-assessment från k3s-dokumentationen (eller kube-bench),
  beta av avvikelser.
- [ ] Default-deny NetworkPolicies per namespace — görs EFTER migreringen så
  vi inte felsöker två saker samtidigt.
- [ ] Testa etcd-restore från S3-snapshot en gång ("backup som inte testats
  finns inte").
- [ ] Konfigurera Longhorn S3 backup target så volymer också har backup, inte
  bara pg-dumpar.
- [ ] Skärp PSA stegvis mot `enforce: restricted` där workloads klarar det.

## Största riskerna

1. **Sealed-secrets-nyckeln** — tappas den är det 35 secrets att om-seala.
   Exportera FÖRST (fas 0, steg 1).
2. **O-backat Longhorn-state** — allt som inte räddas i fas 0 är borta när
   gamla klustret rivs.
3. **Multi-arch-byggen** — utan dem står nya klustret utan egna appar.
   Verifiera i GHCR innan fas 4.
4. **Dubbla cloudflared** — skala alltid ner gamla klustrets tunnel före
   verifiering, annars går trafik slumpvis mot båda klustren.
