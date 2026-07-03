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
| OS | Ubuntu Server 26.04 LTS (alla noder) — reviderat 2026-07-03 från 24.04; Bo installerade 26.04 (giltig nyare LTS) |
| Migreringsstrategi | Nytt kluster + Flux-bootstrap mot samma repo; data återställs från S3-dumpar |
| Datastore | Inbäddad etcd (`cluster-init: true`), snapshots till S3 |
| Arkitektur | Multi-arch-images (amd64 + arm64) så workloads kan köra på M720q |
| Provisionering | Ansible i `ansible/` i detta repo, baserat på k3s-io/k3s-ansible |
| Härdning | k3s CIS hardening guide (protect-kernel-defaults, secrets-encryption, audit-logg, PSA) |
| Ansible become | Passwordless sudo för `bo` på noderna (NOPASSWD sudoers-fil, Bo lägger in); playbooks körs utan `--ask-become-pass` |
| k3s-version | `v1.36.2+k3s1` (stable-kanalen 2026-07-03) — pinnad i ansible |

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

- [x] **[Bo]** Skapa USB-sticka med **Ubuntu Server 24.04 LTS (amd64)** —
  ladda ner ISO, skriv med balenaEtcher eller
  `sudo dd if=ubuntu-24.04-live-server-amd64.iso of=/dev/diskN bs=4m`.
  → Bo installerade **Ubuntu Server 26.04 LTS** (nyare LTS, släppt apr 2026),
  inte 24.04. Avviker från beslutstabellen men 26.04 är en giltig LTS; k3s och
  ansible-rollerna påverkas inte nämnvärt. Justera versionsreferenser i
  ansible/fas 2 därefter.
- [x] **[Bo]** BIOS på M720q (F1 vid boot):
  - **Power → After Power Loss: Power On** (homelab-krav: startar själv efter strömavbrott)
  - Boot order: USB först (tillfälligt)
  - Intel VT-x/VT-d: enabled
  - Secure Boot: kan vara på (Ubuntu stödjer det)
  → Bekräftat av Bo 2026-07-03 (kan ej verifieras via SSH).
- [x] **[Bo]** Installera Ubuntu på **SATA-SSD:n** (välj rätt disk — INTE NVMe:n!):
  - Hostname: `m720q`, användare: `bo`
  - "Install OpenSSH server": JA; importera gärna SSH-nyckel från GitHub
  - Ingen extra snap-paketering behövs
  → Klart: OS på `sda` (SATA-SSD, 238.5G, LVM `ubuntu-vg`, root 100G). NVMe
  (`nvme0n1`, 931.5G) helt tom. **OBS:** installern la bara 100G i root-LV:t —
  ~135G i vg:n är oallokerat, men det spelar ingen roll eftersom etcd/containerd
  + Longhorn hamnar på NVMe:n (fas 2).
- [x] **[Bo]** M720q-IP: **`192.168.50.212`** (nästa efter Pi:erna .210/.211).
  Satt statiskt i Ubuntu-installern (Manual IPv4: 192.168.50.0/24, gateway
  192.168.50.1, DNS 1.1.1.1/8.8.8.8 — INTE Pi-hole, undvik kyckling-och-ägg vid
  boot). Lägg ändå in en DHCP-reservation för .212 i routern så poolen aldrig
  delar ut den. Detta IP ska in i k3s `tls-san` (fas 3) och ansible-inventory.
- [x] **[Bo]** Lägg in claude-boxens pubnyckel så Ansible når maskinen:
  ```bash
  cd ~/Development/apple-container
  ./claude-box.sh ssh-setup pubkey | ssh bo@<m720q-ip> 'cat >> ~/.ssh/authorized_keys'
  ```
  → Bekräftat: `ssh m720q` funkar nyckelbaserat från boxen.
- [x] **[Claude]** Lägg till `m720q` i `~/.ssh/config` i boxen (IP från ovan) och
  verifiera `ssh m720q 'hostname && lsblk'` — kontrollera att NVMe:n syns.
  → Klart 2026-07-03: SSH-config uppdaterad, `ssh m720q` ger hostname `m720q`,
  arch `x86_64`, NVMe 931.5G tom och synlig.

## Fas 2 — Ansible-struktur

- [x] **[Claude]** Skapa `ansible/` i detta repo (egen feature-branch/PR):
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
  → Klart 2026-07-03. Även `nvme-storage`-roll, `group_vars/all/` (main + vault-
  example), `.ansible-lint`, README. Secrets (k3s-token, etcd-S3-creds) i
  gitignorerad `group_vars/all/vault.yml` — `vault.yml.example` committad. Collections
  (ansible.posix 2.2.1, community.general 13.1.0) i `./collections` (gitignorerat).
- [x] **[Claude]** `common`-rollen:
  - ufw: allow 22/tcp (LAN), 6443/tcp (LAN), 10250/tcp + 8472/udp (endast klusternoder),
    servicelb-portar för Mosquitto m.m. från LAN; default deny incoming
  - SSH-härdning: `PasswordAuthentication no`, `PermitRootLogin no`
  - unattended-upgrades
  - `/etc/sysctl.d/90-kubelet.conf` (krävs av `protect-kernel-defaults`):
    `vm.panic_on_oom=0`, `kernel.panic=10`, `kernel.panic_on_oops=1`
  → Klart. ufw-servicelb-portar bekräftade mot repots LoadBalancer-services:
    1883/tcp (mosquitto), 53 tcp+udp (pihole). Traefik 80/443 lämnade stängda
    (Cloudflare Tunnel är primär ingress) — kommenterade i `servicelb_lan_ports`.
    SSH-härdning som drop-in (`99-hardening.conf`) för att vinna över cloud-init.
    Full k3s-sysctl-set (även `vm.overcommit_memory=1` + `kernel.keys.root_max*`
    som k3s protect-kernel-defaults faktiskt kräver, utöver de 3 i listan ovan).
    Hanterar även NOPASSWD-sudoers för `bo` (`/etc/sudoers.d/90-bo-nopasswd`,
    `visudo -cf`-validerad) — idempotent, ger p0/p1 samma i fas 5. Bo satte
    dessutom upp det manuellt på m720q 2026-07-03 för att bootstrappa.
- [x] **[Claude]** Disklayout-tasks (eller engångskörning) för NVMe via LVM:
  - `~200G` → `/var/lib/rancher` (etcd + containerd på NVMe)
  - resten (~730G) → `/var/lib/longhorn`
  - fstab-entries, ext4
  → Klart i `nvme-storage`-rollen (LVM vg `data-vg` på `/dev/nvme0n1`, rancher-LV
    200g + longhorn-LV 100%FREE, ext4, `ansible.posix.mount` = mount+fstab). Körs
    FÖRE k3s-server i site.yml. **VARNING i rollen:** wipe:ar nvme0n1 (bekräftat tom).
- [x] **[Claude]** `k3s-server`-rollen: k3s-binär (pinnad version — slå upp aktuell
  stabil), `/etc/rancher/k3s/config.yaml` + audit-policy + PSA-config
  (se fas 3), systemd-enhet, token-hantering.
  → Klart. Pinnad `v1.36.2+k3s1` (stable-kanalen 2026-07-03). config/psa/audit
    som templates. etcd-S3 aktiveras bara när creds finns (annars lokala snapshots),
    så första bootstrap funkar utan secrets. Token valfri via vault (k3s genererar
    annars). Install via get.k3s.io + `INSTALL_K3S_VERSION`; väntar på Node Ready.
- [x] **[Claude]** Kör `ansible-lint` rent innan PR.
  → Passerar på **production**-profilen (0 failures, 10 filer).

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

- [x] **[Claude]** Kör playbooken mot m720q.
  → Klart 2026-07-03: `ok=32 changed=22 failed=0`. **OBS:** första körningen dog
    direkt på `stdout_callback = yaml` i `ansible.cfg` — den callbacken togs bort
    i community.general 12+ (vi kör 13.1.0). Bytt till `default` + `result_format
    = yaml`. Bo satte upp NOPASSWD-sudoers manuellt (första försöket hade inte
    landat i `/etc/sudoers.d/`); rollen hanterar den nu deklarativt framåt.
    **Andra fyndet (härdningsbugg):** `PasswordAuthentication no` bet inte —
    `sshd -T` visade fortfarande `yes`. sshd använder FÖRSTA värdet per nyckel,
    och Ubuntus `50-cloud-init.conf` (`PasswordAuthentication yes`) läses före
    vår `99-hardening.conf`. Fix: döpt om till `00-hardening.conf` (läses före
    50-, vinner) + tar bort gamla 99-filen. **Applicerat 2026-07-03** efter att
    Bo lagt in sin SSH-nyckel: `sshd -T` = `passwordauthentication no`,
    lösenordsförsök ger nu `Permission denied (publickey)`, nyckel-login (Bo +
    boxen) intakt.
- [x] **[Claude]** Verifiera: `ssh m720q 'sudo k3s kubectl get nodes'`,
  ta manuell etcd-snapshot (`k3s etcd-snapshot save`) och bekräfta S3-uppladdning.
  → Node **Ready**, `v1.36.2+k3s1`, roll `control-plane,etcd`. Verifierat i kraft:
    `secrets-encryption` (Enabled, hashes match), `protect-kernel-defaults`,
    audit.log skrivs (4.1M), **PSA enforce baseline** (privilegierad pod i
    `default` blev Forbidden). NVMe-LVM monterat: `/var/lib/rancher` 196G +
    `/var/lib/longhorn` 719G. Manuell etcd-snapshot sparad **lokalt**
    (`manual-phase3-...`, 1.3M) — **S3-uppladdning väntar på creds** (etcd-s3
    aktiveras automatiskt av config-templaten när `etcd_s3_access_key` fylls i
    vaulten; region eu-north-1, bucket `k12n-homelab-db-backups`). De schemalagda
    snapshotsen (cron 03:00, retention 14) körs av servern. `k3s etcd-snapshot`-
    CLI:t loggar ofarliga "Unknown flag ... skipping" (subkommandot har färre
    flaggor än `k3s server` — servern applicerar dem, bevisat ovan).
- [x] **[Bo]** Hämta kubeconfig till hosten och gör den till context, t.ex.:
  ```bash
  ssh bo@<m720q-ip> 'sudo cat /etc/rancher/k3s/k3s.yaml' \
    | sed 's/127.0.0.1/<m720q-ip>/' > ~/.kube/m720q.yaml
  # merga in i ~/.kube/config som context "homelab-new"
  ```
  Kör sedan `./claude-box.sh kube homelab-new` så Claude når nya klustret.
  → Klart 2026-07-03: `kubectl config current-context` = `homelab-new`, når
    m720q (Node Ready, `v1.36.2+k3s1`).

## Fas 4 — Flux-bootstrap + dataåterställning

Ordningen är viktig — sealed-secrets-nyckeln FÖRE Flux:

- [x] **[Bo]** Återställ nyckeln (host, context `homelab-new`):
  `kubectl apply -f ~/sealed-secrets-keys-backup.yaml`
  → Klart 2026-07-03: 9 `sealed-secrets-key*`-secrets ligger i `kube-system`
    (märkta `sealedsecrets.bitnami.com/sealed-secrets-key`). Nyckeln från gamla
    klustret är på plats FÖRE Flux — sealed secrets kan dekrypteras.
- [x] **[Claude]** Installera flux-operator (Helm) + git-auth-secret för repot,
  applicera rot-`flux.yaml` (FluxInstance).
  → Klart 2026-07-03. flux-operator v0.53.0 via Helm OCI-chart
    (`oci://ghcr.io/controlplaneio-fluxcd/charts/flux-operator`). Alla 7
    controllers uppe. **Git-auth:** `flux.yaml` har `provider: github`, som
    kräver GitHub App-creds (INTE username/password). Bo genererade ny private
    key på den befintliga Flux-appen (App ID `1991306`, Installation ID
    `86929523`); `flux-system`-secreten skapad med
    `githubAppID`/`githubAppInstallationID`/`githubAppPrivateKey`, `.pem` raderad
    efteråt (gitignorerad). GitRepository hämtar `refs/heads/main` OK.
- [ ] **[Claude]** Följ utrullningen (`flux get kustomizations`), bekräfta att
  sealed secrets dekrypteras: `kubectl get sealedsecrets -A` utan fel i status.
  → **Två bootstrap-blockerare hittade och lösta 2026-07-03:**
    1. **Chart-repot flyttat:** `gitops/infrastructure/sources/sealed-secrets.yaml`
       pekade på `https://bitnami-labs.github.io/sealed-secrets` (nu 404 —
       GitHub Pages-siten borttagen). Bytt till `https://bitnami.github.io/sealed-secrets`
       (chart 2.17.7 / appVersion 0.32.2 finns där). Fix i PR (denna branch).
    2. **CRD-moment-22:** `infrastructure-controllers` innehåller både
       sealed-secrets-HelmReleasen OCH SealedSecret-CR:er (t.ex.
       cloudflare-tunnel). På ett tomt kluster faller hela kustomizationens
       dry-run på `no matches for kind "SealedSecret"` → controllern som skapar
       CRD:n hinner aldrig appliceras. Löst genom att applicera just
       sealed-secrets-HelmReleasen manuellt en gång (`kubectl apply -f
       .../sealed-secrets/helmrelease.yaml`); Flux adopterar den sen utan drift.
       Controllern (v0.32.2) registrerade alla 9 återställda nycklar → sealed
       secrets dekrypteras. (Framtida fresh-bootstraps: överväg att flytta
       sealed-secrets till `infrastructure-crds` så CRD:n finns före CR:erna.)
    3. **DNS-deadlock (pihole hostPort :53 kapar nodens egen DNS):** k3s
       ServiceLB (klipper) binder hostPort 53 på noden för pihole:s
       LoadBalancer-service (`pihole-dns`). Med `net.ipv4.conf.all.route_localnet=1`
       (kube-proxy-default) DNAT:as ALLA `udp/tcp dport 53` — inklusive nodens
       frågor till systemd-resolved-stubben `127.0.0.53:53` — till pihole-poden.
       På ett kallt kluster körs pihole inte (ImagePullBackOff) → nodens DNS ger
       "connection refused" → containerd kan inte pulla NÅGRA images (cert-manager,
       traefik, grafana, cloudflared, prometheus, pihole själv) → deadlock.
       Symptom: `getent hosts ghcr.io` = FAIL men `resolvectl query` (D-Bus,
       bypassar :53) funkar. **Löst engångsvis:** `flux suspend kustomization
       pihole` + `kubectl delete svc pihole-dns -n pihole` → svclb släpper :53 →
       stub-DNS (→1.1.1.1) funkar → alla images pullas (pihole-imagen cachas på
       noden) → pihole-poden startar → `flux resume kustomization pihole` sist.
       **Varför bara ett engångsproblem:** efter första pullen är alla images
       cachade i containerd, så vid reboot (t.ex. BIOS auto-power-on) startar
       pihole från cache utan registry-DNS, varefter nodens DNS går via en körande
       pihole. Kall bootstrap med tom image-cache är enda tillfället deadlocken slår.
    4. **k3s:s bundlade Traefik krockar med Flux-traefiken:** repot kör egen
       Traefik via Flux-HelmRelease (namespace `traefik`, chart 38.0.1), men
       k3s-config-templaten disable:ade inte k3s egen Traefik → k3s installerar
       sin bundlade (kube-system, helm.cattle.io) som slåss om Traefik-CRD:erna
       och crashloopar `helm-install-traefik`-jobbet. Fix: `disable: [traefik]`
       i `ansible/roles/k3s-server/templates/config.yaml.j2` (ServiceLB behålls).
       Live-städning (k3s-omstart som avinstallerar addon:et) görs efter att
       trädet stabiliserat sig.
  → **Ytterligare tre fynd 2026-07-03 (fas 3 PSA + fas 0 nodeSelector-arv + grafana-RBAC):**
    5. **PSA `enforce: baseline` blockerar redpanda-v2 och homebridge:** fas 3
       satte global PSA baseline med exemptions bara för `kube-system` +
       `longhorn-system`. Men redpanda:s `tuning`-init-container (privileged +
       SYS_RESOURCE) och homebridge (`hostNetwork: true` + hostPorts 51826/8581)
       bryter mot baseline → poddarna kunde inte skapas (StatefulSet/ReplicaSet
       `FailedCreate ... violates PodSecurity "baseline:latest"`). Fix: label
       `pod-security.kubernetes.io/enforce: privileged` på båda namespacen
       (`gitops/apps/base/{redpanda-v2,homebridge}/namespace.yaml`). Säkert i
       shared base — labeln bara relaxar PSA, och gamla klustret enforcar ingen
       PSA. Applicerat live för att låsa upp direkt; PR för durabilitet.
    6. **Prometheus `nodeSelector: {hostname: p1}` (LÄMNAD kvar med flit):**
       PR #133 pinnade Prometheus till p1 för att skydda gamla klustrets p0.
       På nya (bara m720q) → Prometheus Pending, vilket håller `prometheus →
       loki → alloy` icke-Ready. **Får INTE tas bort från shared base nu** —
       gamla klustret syncar samma main och skulle då lägga Prometheus på p0
       (96% CPU → redpanda-raft-strul). Tas bort som en del av fas 5-cutovern
       (se fas 0-noten). Monitoring-stacken förblir alltså medvetet Pending på
       nya klustret tills dess. Allt annat (auth, appar, dataväg) är opåverkat.
    7. **grafana datasource-generator RBAC — `create` + `resourceNames`:** jobbet
       `grafana-datasource-configmap-generator` (injicerar TimescaleDB-datasource
       i en configmap) nekades `configmaps is forbidden ... cannot create`. Role:n
       hade `resourceNames: [grafana-timescaledb-datasource]` PÅ SAMMA regel som
       `create` — och k8s RBAC ignorerar resourceNames för `create` (namnet är
       okänt vid admission), så create auktoriserades aldrig. På gamla klustret
       fanns configmapen redan → bara `update` behövdes. Fix: bröt ut `create`
       till en egen oscopad regel, behöll named get/update/patch
       (`gitops/apps/base/grafana/datasource-generator-rbac.yaml`). Applicerat
       live + PR.
- [x] **[Claude]** Återställ databaser från S3-dumpar (engångs-restore via
  `kubectl exec psql < dump` är OK — det är migrations, inte restores, som är
  GitOps): timescaledb, homelab-settings, backstage. Verifiera radantal mot
  förväntan.
  → Klart 2026-07-03. Bo la upp färska dumpar samma kväll (de dagliga
    cronjob-dumparna före dess var 20-byte-trasiga, jfr fas 0-noten). Restore
    kördes via engångs-poddar (`postgres:16` + awscli, S3-creds återanvända från
    respektive backup-secret; manifesten i scratchpad). Mönster: skala ner
    writers → `DROP DATABASE ... WITH (FORCE)` + `CREATE` → restore in i tom DB →
    skala upp. **0 fel** i alla tre.
    - **timescaledb** (`telemetry-20260703-195704.sql.gz`, 116 MB gz / 825 MB):
      plain `pg_dump` UTAN pre_restore-wrapping (dumpar `_timescaledb_internal`-
      chunkar som råa tabeller), så restore kördes i `timescaledb_pre_restore()`-
      läge in i en nydroppad DB (migreringsjobbets schema hade andra chunk-ID:n)
      + `timescaledb_post_restore()` efteråt. Writers `timescaledb/redpanda-connect`
      (sinken) + `spotprice/spotprice-api` nedskalade under tiden. Resultat:
      energy_consumption **7 606 846** rader (2026-04-02 → 2026-07-03 19:57, ända
      fram till backuptidpunkten), heatpump_status 221 903, temperature_sensors
      1 420, spot_prices 1 632, apns_device_tokens 1, schema_migrations 7. 4
      hypertables + 2 continuous aggregates (energy_daily_summary, energy_hourly)
      med retention/refresh-policies aktiva.
    - **homelab-settings** (`homelab_settings-20260703-200350.sql.gz`, 38 KB):
      plain `pg_dump`. Writers api + outbox-processor + redpanda-connect
      nedskalade. Resultat: outbox 1448 (703 published, 745 confirmed, inga
      pending/failed), power_plugs 2, power_plug_schedules 5, settings 1,
      schema_migrations 2.
    - **backstage** (`backstage-20260703-195109.sql.gz`, 3.8 MB): `pg_dumpall`
      (PR #132) — 13 databaser (backstage + 12 plugin-DB:er). Droppade befintliga
      backstage*-DB:er, matade dumpall mot `postgres`-DB:n som superuser
      `backstage`. Enda ERROR-raden var väntad `role "backstage" already exists`
      (rollen finns via sealed secret). Alla 13 DB:er återställda (7-12 MB st),
      catalog `final_entities` 15. App Ready 1/1 efteråt.
    Alla appar uppskalade och Running efteråt; settings-api/outbox anslöt rent
    till DB. **Obs (orelaterat):** settings-consumern loggar ännu
    `UnknownTopicOrPartition` för `homelab-heatpump-telemetry` — Redpanda-topicen
    är inte skapad på nya klustret ännu (rpk-topic-jobbet), separat från restoren.
- [x] **[Claude]** Återställ räddat state från fas 0 (Homebridge, Authelia, Pi-hole).
  → Klart 2026-07-03. Tarbollarna från `pre-migration-state/20260703/` extraherade
    in i respektive RWO-PVC via helper-poddar (`postgres:16`, root). Sekvens per app:
    kopiera S3-creds temporärt till namespacet (jq-klon av `timescaledb-backup-aws`
    → `s3-restore-creds`, raderad efteråt) → skala app→0 → vänta på Longhorn-detach
    → mounta PVC:n i helper → wipe (behåll `lost+found`) + `tar --strip-components`
    → chown vid behov → skala upp. Grafana/Authelia/Pi-hole ärver global PSA
    `baseline` (root-pod OK); homebridge-ns är `privileged`. **Helper-poddarna fick
    egen DNS (`dnsConfig` 1.1.1.1)** eftersom pihole-nedskalningen tillfälligt bryter
    nodens/coredns :53 (svclb-deadlocken, fix #3) — annars hade apt/aws inte kunnat
    resolva under fönstret. Resultat (alla appar Ready efteråt, kluster-DNS friskt):
    - **authelia** (`data/` → /data, chown 1000): db.sqlite3 311K→**790K** (TOTP-
      registreringar tillbaka); "Storage schema is already up to date", inga fel.
    - **pihole** (`etc/pihole/` → /etc/pihole, strip 2): gravity **83 809**
      blockdomäner, 1 adlist, **26** lokala DNS-poster (custom.list), pihole.toml
      (67K) återställd. `pihole-FTL.db` (query-stats) korrekt exkluderad, återskapas.
      Ready, FTL kör som uid 1000, blocking enabled.
    - **grafana** (`grafana/` → /var/lib/grafana, chown 472): grafana.db →**2.1M**;
      DB-migreringar rena (performed=0 skipped=572 = redan rätt schema, versions-
      kompatibel). "database is locked"-retries vid provisioning är ofarliga.
    - **homebridge** (`homebridge/` → /homebridge, 236M inkl. node_modules):
      persist/ (HomeKit-identitet `CC223DE3CE30`) + config.json + .uix-secrets +
      accessories/ återställda. `/var/lib/homebridge` är symlink → /homebridge, så
      PVC-pathen stämmer. **Homebridge v1.11.4 kör på 51826**, mqttthing-tillbehören
      laddar. **Obs:** plugin `homebridge-mqttthing` varnar att den kräver Node
      ≤22 men imagen kör v24 — bara en engine-varning, pluginet laddar och funkar;
      värt att hålla ögonen på vid framtida homebridge/Node-bump.
    Detta var en engångssnapshot (fas 0) — HomeKit-parningar/TOTP som skapats på
    GAMLA klustret EFTER 2026-07-03 06:5x fångas inte; gör en färsk räddning strax
    före fas 5-cutovern om något nytt tillkommit.
- [x] **[Claude]** Re-seala actions-runnerns kubeconfig — gamla innehållet pekar
  på p1.local:6443. Generera ny mot m720q, seala med `kubeseal --fetch-cert`
  mot nya klustret, committa.
  → Klart 2026-07-03. SealedSecret `github-actions-kubeconfig` i ns
    `actions-runners` (nyckel `KUBECONFIG_DATA`, monteras på `/etc/kubeconfig/config`
    i runnern). Kustomizationen definierar ingen egen SA/RBAC för identiteten, och
    det gamla `p1.local:6443`-innehållet var alltså admin-kubeconfig:en — så jag
    genererade nya klustrets admin-kubeconfig (`kubectl config view --raw --minify
    --flatten` på `homelab-new`-contexten; server redan `https://192.168.50.212:6443`)
    och sealade om den. Sealat mot nya klustrets controller
    (`--controller-name sealed-secrets --controller-namespace kube-system`, v0.32.2,
    INTE p1). Verifierat: applicerad live → controller-event "SealedSecret unsealed
    successfully", dekrypterad kubeconfig pekar på .212:6443 och funkar (`get nodes`
    = m720q Ready, `auth can-i patch deployments` = yes). Bara ciphertext-raden
    ändrad i filen; metadata/template orört. Plaintext-kubeconfig:en raderades ur
    scratchpad, aldrig committad. **Obs (säkerhet):** identiteten är cluster-admin,
    precis som förr — inga workflows kör kubectl idag (bara bygg/push), men om det
    införs vore en scopad ServiceAccount-token (bara deployments patch/restart)
    säkrare än admin i CI-monterad secret. Lämnad som ev. fas 6-härdning.
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
