# AI Architectural Review

Every pull request is reviewed by an AI agent against the rules in
[CLAUDE.md](../CLAUDE.md). The review is a guardrail next to the automated checks
(`Validate`, the app test workflows); it does not replace reading the diff.

## How it works

`.github/workflows/arch-review.yml` runs on every PR that is opened, updated or reopened.
It calls `tools/arch-review/review_pr.py`, which:

1. Diffs the PR's base and head commits. Each file's diff is capped at 50,000
   characters.
2. Sends the diffs, the PR title and description, and the full text of CLAUDE.md to
   Claude. The model is set in the script (`model=` in `review_changes`).
3. Posts one summary comment on the PR.
4. Exits non-zero when the review recommends **REQUEST CHANGES**, which marks the
   `architectural-review` check as failed.

Each new push adds a new comment; older comments stay.

## What it looks at

| Area | Examples |
|---|---|
| Application code | homelab-api stays read-only, auth patterns, over-engineering, injection risks |
| Dockerfiles | Multi-stage builds with dependency caching |
| Tests | Coverage for critical paths: data writes, auth, message processing |
| GitOps manifests | SealedSecrets instead of plain Secrets, resource limits, namespaces |
| Migrations | File structure and the `schema_migrations` insert |
| Workflow | Feature branches, commit message format |

The rules come from CLAUDE.md only. To change what the review enforces, change
CLAUDE.md.

## Reading the comment

```markdown
## 🤖 AI Architectural Review

### ✅ Compliant
- Dockerfile uses layered builds with dependency caching

### ❌ Violations (BLOCKING)
- **File**: `applications/homelab-api/src/handlers/energy.rs:145`
  **Violation**: Database INSERT operation detected
  **Principle**: homelab-api must be read-only
  **Suggestion**: Write through the Redpanda Connect sink instead

### ⚠️ Warnings
- Consider adding integration tests for the new Kafka consumer logic

### 📊 Summary
- **Violations**: 1
- **Warnings**: 1
- **Recommendation**: REQUEST CHANGES
```

Violations fail the check. Warnings are advice.

## Limits

- The reviewer only sees the diff between the PR's base and head commits. If the branch
  is behind `main`, changes that landed on `main` in the meantime show up as if the PR
  reverted them. Update the branch and the next review is correct.
- It can be wrong. It has, for example, miscounted the length of a checksum. Check a
  blocking finding against the actual CI result before acting on it.
- The failed check only blocks a merge if `architectural-review` is a required status
  check in the branch protection settings.

## Skipping

Put `[skip arch-review]` in the PR title. The workflow still runs but exits without
calling the model.

## Setup

| Item | Value |
|---|---|
| Secret | `ANTHROPIC_API_KEY` in the repository secrets |
| Permissions | `contents: read`, `pull-requests: write` |
| Dependencies | `tools/arch-review/requirements.txt` |

If no comment appears, open the workflow run under the Actions tab; a missing or invalid
API key is the usual cause.
