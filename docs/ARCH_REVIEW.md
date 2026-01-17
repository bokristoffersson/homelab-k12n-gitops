# AI Architectural Review

## Overview

Every PR is automatically reviewed by an AI agent (Claude Sonnet 4.5) for compliance with architectural principles defined in CLAUDE.md.

## What Gets Reviewed

**Code changes (applications/*)**:
- Database operations (read-only enforcement for homelab-api)
- Authentication patterns (JWT validation)
- Over-engineering detection (unnecessary abstractions, features not requested)
- Security vulnerabilities (SQL injection, XSS, command injection)

**Dockerfiles (applications/*/Dockerfile)**:
- Multi-stage builds with dependency caching
- Rust: Cargo.toml copied before src/, dummy lib.rs + main.rs for caching
- Node.js: package.json copied before source code
- Required build dependencies (cmake, libssl-dev for rdkafka)

**Tests (applications/*/tests/)**:
- Coverage for critical paths (Kafka consumers, data writes, auth)
- New features touching critical paths include tests
- Test coverage appropriate for risk level

**GitOps configs (gitops/**)**:
- Proper namespace usage
- Sealed secrets (not plain Secret manifests)
- Resource limits and requests defined
- FluxCD best practices

## Review Process

1. **PR opened/updated** → Workflow triggered (`.github/workflows/arch-review.yml`)
2. **Agent analyzes** changed files against CLAUDE.md principles
3. **Agent posts** single summary comment with:
   - ✅ Compliant aspects
   - ❌ Violations (blocks merge)
   - ⚠️ Warnings (advisory)
4. **If violations found** → PR check fails (blocks merge)
5. **Fix violations** → Push changes → Agent re-reviews automatically

## Example Review Comment

```markdown
## 🤖 AI Architectural Review

### ✅ Compliant
- Dockerfile uses proper layered builds with dependency caching
- JWT validation correctly implemented in auth middleware

### ❌ Violations (BLOCKING)
- **File**: `applications/homelab-api/src/handlers/energy.rs:145`
  **Violation**: Database INSERT operation detected
  **Principle**: homelab-api must be read-only (CLAUDE.md: Rust/Backend section)
  **Suggestion**: Move write operations to redpanda-sink service

### ⚠️ Warnings
- Consider adding integration tests for new Kafka consumer logic

### 📊 Summary
- **Total files changed**: 3
- **Violations**: 1
- **Warnings**: 1
- **Recommendation**: REQUEST CHANGES
```

## Cost

- **Per PR**: ~$0.01-0.03 (depends on change size)
- **Monthly estimate** (20 PRs): ~$0.50
- **Model**: Claude Sonnet 4.5

## Skipping Review for Specific PRs

Add `[skip arch-review]` to PR title to bypass review (use sparingly for urgent hotfixes or documentation-only changes).

**Example**: `[skip arch-review] docs: update README`

## Configuration

**Required GitHub Secret**:
- `ANTHROPIC_API_KEY` - Claude API key for reviews

**Workflow triggers**:
- Pull request opened
- Pull request synchronized (new commits pushed)
- Pull request reopened

**Permissions**:
- `contents: read` - Read repository files
- `pull-requests: write` - Post review comments
- `checks: write` - Set PR check status

## Troubleshooting

**Review not running?**
- Check GitHub Actions tab for workflow errors
- Verify `ANTHROPIC_API_KEY` is set in repository secrets
- Ensure workflow file exists at `.github/workflows/arch-review.yml`

**False positives?**
- Agent provides detailed explanations for each violation
- If incorrect, add `[skip arch-review]` to PR title and proceed manually
- Consider updating CLAUDE.md if principle needs clarification

**Review taking too long?**
- Typical review: 10-20 seconds
- Large PRs (50+ files): up to 60 seconds
- If timeout occurs, check Anthropic API status

## How It Works

1. **Fetch changed files** - Git diff between base and head refs
2. **Read principles** - Load CLAUDE.md architectural guidelines
3. **Build context** - Combine file diffs + principles + PR metadata
4. **AI analysis** - Send to Claude Sonnet 4.5 with structured prompt
5. **Parse response** - Extract violations, warnings, compliant aspects
6. **Post comment** - Single summary comment on PR
7. **Set status** - Success (✅) or failure (❌) check status
8. **Block/allow merge** - Exit code determines if PR can merge

## Benefits

- **Prevents architectural drift** - Catches violations before merge
- **Educational** - Developers learn principles through review feedback
- **Consistent** - Same standards applied to all PRs
- **Low maintenance** - Update CLAUDE.md, no code changes needed
- **Cost-effective** - ~$0.50/month for comprehensive reviews

## Limitations

- **Not a replacement for human review** - AI assists, doesn't replace code review
- **Context limits** - Very large PRs (100+ files) may hit token limits
- **Interpretation variance** - AI may occasionally miss edge cases
- **Best for homelab scale** - Optimized for small team, moderate PR volume

## Contributing

To improve the AI review system:

1. **Update principles** - Edit CLAUDE.md to add/clarify guidelines
2. **Refine prompts** - Modify `tools/arch-review/review_pr.py` prompt section
3. **Adjust sensitivity** - Update violation detection logic in script
4. **Test changes** - Create test PR with `[skip arch-review]` to verify manually

## Support

- **Issues**: GitHub Issues on this repository
- **Documentation**: See CLAUDE.md for architectural principles
- **Logs**: Check GitHub Actions workflow logs for debugging
