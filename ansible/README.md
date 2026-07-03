# Ansible — M720q homelab cluster provisioning

Provisions the hardened k3s cluster for the Pi→M720q migration
(see `docs/MIGRATION_M720Q.md`). Based on the k3s CIS hardening guide.

## Layout

```
ansible.cfg              # collections_path=./collections, passwordless-sudo become
inventory.yml            # m720q in [server]; p0/p1 join [agent] in phase 5 (IPs, not .local)
requirements.yml         # galaxy collections (ansible.posix, community.general)
site.yml                 # common → nvme-storage → k3s-server → k3s-agent
group_vars/all/
  main.yml               # non-secret vars (k3s version, IPs, firewall ports)
  vault.yml.example      # template for secrets (k3s token, etcd S3 creds)
roles/
  common/                # ufw, SSH hardening, unattended-upgrades, protect-kernel sysctls
  nvme-storage/          # LVM on NVMe: /var/lib/rancher + /var/lib/longhorn (runs before k3s)
  k3s-server/            # hardened config.yaml + PSA + audit policy + install
  k3s-agent/             # agent config + install (phase 5)
```

## Prerequisites

- `bo` has passwordless sudo (NOPASSWD sudoers) on each node.
- The claude-box SSH key (`~/.ssh/id_p1`) is in each node's `authorized_keys`.

## Usage

```bash
cd ansible
ansible-galaxy collection install -r requirements.yml   # into ./collections
cp group_vars/all/vault.yml.example group_vars/all/vault.yml
$EDITOR group_vars/all/vault.yml                        # set token + S3 creds
ansible-vault encrypt group_vars/all/vault.yml

ansible-lint
ansible-playbook site.yml --check --ask-vault-pass      # dry run
ansible-playbook site.yml --ask-vault-pass              # apply
```

> `nvme-storage` wipes `/dev/nvme0n1` into an LVM volume group. It was confirmed
> empty on m720q before this role was written. Double-check on any other host.
