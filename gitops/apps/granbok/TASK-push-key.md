# Task: give granbok the key it sends push with

Written 2026-11-25 from the granbok repo for the agent that maintains this cluster. Delete this
file in the same pull request that completes the task.

## What it is for

The granbok core sends push to the granbok iOS app (bundle id `se.granskogen.granbok`) through
Apple's service, about proposals that wait and returns that are due. It signs its calls with a
key from Bo's Apple developer account. Without the key nothing is sent and everything else
works.

The requirements are in the granbok repo: `deploy/HOMELAB.md`, "Push till appen".

## Before starting

1. **The granbok version must have push.** It came after 0.9.0 (granbok pull request 9); ask Bo
   which tag. On 0.9.0 or older the variables below are ignored. Do the upgrade in
   `TASK-upgrade-0.9.0.md` first if the cluster is still on 0.5.0.
2. **Ask Bo for three things:**
   - the key file `AuthKey_<key id>.p8` from Apple. It is a secret and can be downloaded from
     Apple only once. Never commit it, paste it in a pull request or leave it in the working
     tree.
   - the key's id, ten characters (also in the file's name)
   - the developer team's id, ten characters

   The key has to be good for both Apple's sandbox and production. `spotprice-api` already sends
   push with a key in the secret `spotprice-apns`. If Bo says it is the same team and that key is
   not limited to the other app, the same key works here; a key of granbok's own can be revoked
   on its own. Either way granbok gets a sealed secret of its own in its own namespace.

## Steps

The pattern is the one `gitops/apps/spotprice/` uses: the key as a file in a sealed secret,
mounted read-only.

### 1. Sealed secret `granbok-push` in namespace `granbok`

Keys: `AuthKey.p8` (the file, unchanged), `KEY_ID`, `TEAM_ID`. Seal it as `CLAUDE.md` describes
under Sealed Secrets, with `--from-file=AuthKey.p8=<path>` for the file, write
`gitops/apps/granbok/granbok-push-sealed.yaml` and add it to `kustomization.yaml`. Delete the
plain file from wherever Bo put it once it is sealed.

### 2. `core.yaml`

- mount the secret's `AuthKey.p8` read-only at `/push` (a `volumes` entry with `items`, and a
  `volumeMounts` entry). The container runs as user 10001 with a read-only root file system;
  a secret volume is readable by it as it is.
- add the environment:

| Variable | Value |
|---|---|
| `GRANBOK_PUSH_KEY_FILE` | `/push/AuthKey.p8` |
| `GRANBOK_PUSH_KEY_ID` | from the secret, key `KEY_ID` |
| `GRANBOK_PUSH_TEAM_ID` | from the secret, key `TEAM_ID` |

`GRANBOK_PUSH_TOPIC` is not needed; it defaults to `se.granskogen.granbok`.

Nothing changes for the agent, the gateway or the network policies. The core reaches
`api.push.apple.com` and `api.sandbox.push.apple.com` on 443; egress is not restricted today.

## Check

1. The core starts. If the key file cannot be read, or an id is missing, it does not start and
   its log says which. The core is the bookkeeping's web interface too, so watch it come up.
2. Bo logs in in the app on his phone and allows notifications. A receipt he hands in gives a
   notification within a couple of minutes of the proposal being made.
3. If none comes: the core logs a warning with Apple's reason when a message is refused.
   `InvalidProviderToken` means the key id or team id is wrong, `BadDeviceToken` that the app
   was built for the other environment than it registered with, `TopicDisallowed` that the key
   is not good for this app.

## When done

Add the push key to the granbok entry in `CLAUDE.md` and delete this file.
