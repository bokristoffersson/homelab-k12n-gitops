# Task: upgrade granbok from 0.5.0 to 0.9.0 and let the iOS app log in

Written 2026-11-25 from the granbok repo for the agent that maintains this cluster. It replaces
the earlier note `TASK-chat-per-account.md`, which was never committed and covered only the first
of these versions. Delete this file in the same pull request that completes the task.

## What changes

Four versions in one step. The requirements are in the granbok repo, tag `v0.9.0`:
`deploy/HOMELAB.md` (the environment tables, "Appens inloggning", "Uppgradera").

- 0.6.0: the agent no longer has a configured owner. It works for whoever has linked their chat
  to their user in the web interface, and settings belong to each account.
- 0.6.0 and 0.7.0: roles and permissions, statement rows as proposals. Nothing to configure.
- 0.8.0: the core reads the receipt mailbox, not the agent. The mailbox variables move from the
  agent's secret to the core's.
- 0.9.0: an API for the iOS app under `/app/v1`, off until `GRANBOK_APP_AUDIENCE` is set.

The database schema goes from version 15 to 23; the migrations need nothing prepared. The
Authelia client `granbok-app` and its claims policy came with the pull request that added this
file.

## Before starting: ask Bo

1. **The receipt mailbox must accept plus addressing.** Each account gets its own address in the
   mailbox, with a secret part after a plus sign. Gmail does this. Bo has to start forwarding
   receipts to his new address after the upgrade; mail to the plain address is no longer opened.
2. **Whether to set models per task.** Optional, see step 3.

## Steps

Confirm that last night's backup completed before anything else. The upgrade cannot be backed
out: an older version does not start against the newer schema.

Check where each variable actually is before editing: some may be plain `env` entries in
`core.yaml` or `agent.yaml` rather than in the secrets.

### 1. Secret `granbok-core` (sealed)

- rename `GRANBOK_OWNER_CHAT_PASSWORD` to `GRANBOK_AGENT_CHAT_PASSWORD`, same value
- add `GRANBOK_MAILBOX_HOST`, `GRANBOK_MAILBOX_PORT`, `GRANBOK_MAILBOX_USERNAME` and
  `GRANBOK_MAILBOX_PASSWORD` with the values they have in the agent's secret today
- keep `GRANBOK_OWNER_USERNAME` exactly as it is. It decided who got the existing books in an
  earlier migration, and Flyway still reads it on every start.
- everything else unchanged

### 2. Secret `granbok-agent` (sealed)

- rename `GRANBOK_OWNER_CHAT_PASSWORD` to `GRANBOK_AGENT_CHAT_PASSWORD`, same value as the core's
- remove `GRANBOK_OWNER_USERNAME`, `GRANBOK_OWNER_SLACK_USER_ID`, the four `GRANBOK_MAILBOX_*`
  variables that moved to the core, and `GRANBOK_MAILBOX_ALLOWED_SENDERS`; the agent reads none
  of them any more. Note the current value of `GRANBOK_MAILBOX_ALLOWED_SENDERS` for Bo first if
  it can be read: he enters those addresses in the web interface afterwards.

### 3. Plain settings

| Where | Variable | Value |
|---|---|---|
| `core.yaml` | `GRANBOK_RECEIPT_MAILBOX` | the address of the receipt mailbox (the same mailbox as `GRANBOK_MAILBOX_USERNAME`) |
| `core.yaml` | `GRANBOK_APP_AUDIENCE` | `https://granbok.k12n.com`, exactly the `audience` of the Authelia client `granbok-app` |
| `agent.yaml` | `GRANBOK_WEB_URL` | `https://granbok.k12n.com` |

Remove `GRANBOK_PROPOSAL_FLAG_ABOVE` from the core if it is set anywhere; it is no longer read.

Optional, only if Bo asks for it: `GRANBOK_AGENT_RECEIPT_MODEL`, `GRANBOK_AGENT_ASSISTANT_MODEL`
and `GRANBOK_AGENT_ONBOARDING_MODEL` in `agent.yaml` set a model per task; without them every
task uses the default model.

Update the comments in `core.yaml` and `agent.yaml` that list what each secret holds and that
say the agent reaches IMAP.

### 4. Images

Set both images to `0.9.0` (`granbok-core` and `granbok-agent`) and roll them out together.

### 5. Authelia

Authelia reads its configuration at start. If it has not been restarted since the client
`granbok-app` was merged, restart it, and check that it comes up: a configuration it does not
accept keeps it from starting, and every login in the homelab goes through it.

### 6. What Bo does afterwards (not the agent's work, but the rollout is not done without it)

Until Bo has done the first of these the agent answers him only with how to link his chat, and
sends no reminders.

1. Log in at `https://granbok.k12n.com`, open "Koppla chatt", fetch a code and send it to the
   bot in a direct message in Slack. The bot answers that the chat is linked.
2. Open "Inställningar" and add the addresses he forwards receipts from as allowed senders.
3. Forward receipts to the receipt address shown on that page from now on.
4. Log in in the iOS app from the home network.

## Check

1. The core's log contains `Started GranbokApplication` and the schema is at version 23.
2. The core's log has no errors about the mailbox, and the agent's none about Slack or the model.
3. A direct message to the bot from Bo before linking gets the answer that names
   `https://granbok.k12n.com/channels`. After Bo's step 1 the bot answers normally.
4. `https://granbok.k12n.com/app/v1/me` without a token answers 401. A redirect to the login
   page means the core is older than 0.9.0.
5. After Bo's step 4 the app shows his to-do list. If the app says granbok did not accept the
   login, the core answered 401 and the reason is in the `WWW-Authenticate` header of that
   answer: `GRANBOK_APP_AUDIENCE` is not set (the answer says the app's API is off), or
   the access token lacks the audience or `preferred_username`. This part of the Authelia configuration is written from Authelia's
   documentation and has not been tried before.

## What must not change

The network policies, the gateway and the rules for the namespace in `CLAUDE.md`. The agent still
reaches the core only inside the cluster, and `/api/` stays blocked in the gateway. `/app/` is
reached through the gateway as the web interface is, from the LAN only.

## When done

Update the granbok entry in `CLAUDE.md` (the app's client, that the core reads the mailbox) and
the list of recent changes there, and delete this file.
