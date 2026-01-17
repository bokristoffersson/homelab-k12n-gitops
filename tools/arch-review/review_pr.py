#!/usr/bin/env python3
"""
AI-powered architectural review for PRs.
Reads CLAUDE.md principles and analyzes changed files.
"""

import os
import sys
from anthropic import Anthropic
from github import Github
import git

# Configuration
ANTHROPIC_API_KEY = os.environ["ANTHROPIC_API_KEY"]
GITHUB_TOKEN = os.environ["GITHUB_TOKEN"]
REPO_NAME = os.environ["GITHUB_REPOSITORY"]
PR_NUMBER = int(os.environ["PR_NUMBER"])


def get_changed_files(repo_path, base_ref, head_ref):
    """Get list of changed files with diffs."""
    repo = git.Repo(repo_path)

    # Get list of changed files
    diff_index = repo.commit(base_ref).diff(repo.commit(head_ref))

    changed_files = []
    for diff_item in diff_index:
        # Get the file path (handle renames)
        file_path = diff_item.b_path if diff_item.b_path else diff_item.a_path

        # Get change type
        if diff_item.new_file:
            status = "added"
        elif diff_item.deleted_file:
            status = "deleted"
        elif diff_item.renamed_file:
            status = "renamed"
        else:
            status = "modified"

        # Get diff text
        try:
            diff_text = repo.git.diff(f"{base_ref}..{head_ref}", "--", file_path)
        except:
            diff_text = "(Binary file or diff unavailable)"

        changed_files.append({
            "path": file_path,
            "status": status,
            "diff": diff_text[:5000]  # Limit diff size to avoid token limits
        })

    return changed_files


def read_claude_md():
    """Read architectural principles from CLAUDE.md."""
    claude_md_path = os.path.join(os.getcwd(), "CLAUDE.md")
    with open(claude_md_path, "r") as f:
        return f.read()


def format_changed_files(changed_files):
    """Format changed files for the prompt."""
    if not changed_files:
        return "No files changed in this PR."

    formatted = []
    for file_info in changed_files:
        formatted.append(f"""
File: {file_info['path']}
Status: {file_info['status']}

Diff:
{file_info['diff']}
---
""")

    return "\n".join(formatted)


def review_changes(changed_files, principles, pr_title, pr_description):
    """Send changes to Claude for architectural review."""
    client = Anthropic(api_key=ANTHROPIC_API_KEY)

    # Build prompt
    prompt = f"""You are an architectural reviewer for a homelab Kubernetes GitOps project.

ARCHITECTURAL PRINCIPLES (from CLAUDE.md):
{principles}

PR CONTEXT:
Title: {pr_title}
Description: {pr_description or "(No description provided)"}

CHANGED FILES IN THIS PR:
{format_changed_files(changed_files)}

YOUR TASK:
Review these changes for compliance with the architectural principles.

ANALYSIS SCOPE:
1. Code changes (applications/*): Check for violations like:
   - Database writes in read-only APIs (homelab-api)
   - Missing JWT validation
   - Over-engineering (unnecessary abstractions, features not requested)
   - Security vulnerabilities (SQL injection, XSS, command injection)

2. Dockerfiles (applications/*/Dockerfile): Verify:
   - Multi-stage builds with dependency caching
   - Rust: Cargo.toml copied before src/, dummy lib.rs + main.rs for caching
   - Node.js: package.json copied before source code
   - Required build dependencies (cmake, libssl-dev for rdkafka)

3. Tests (applications/*/tests/): Ensure:
   - Critical services have tests (Kafka consumers, data writes, auth)
   - New features touching critical paths include tests
   - Test coverage appropriate for risk level

4. GitOps configs (gitops/**): Check for:
   - Proper namespace usage
   - Sealed secrets (not plain Secret manifests)
   - Resource limits and requests defined
   - FluxCD best practices

RESPOND in this exact format:

## Architectural Review

### ✅ Compliant
[List aspects that follow principles correctly. Be specific about what was done right.]

### ❌ Violations (BLOCKING)
[List architectural violations that must be fixed before merge. If none, write "None found."]

Format each violation as:
- **File**: `path/to/file:line`
  **Violation**: [clear description]
  **Principle**: [which principle from CLAUDE.md was violated]
  **Suggestion**: [specific fix recommendation]

### ⚠️ Warnings
[List concerns that should be addressed but don't block merge. If none, write "None."]

### 📊 Summary
- **Total files changed**: X
- **Violations**: X
- **Warnings**: X
- **Recommendation**: APPROVE / REQUEST CHANGES

Be thorough but concise. Focus on actual architectural issues, not style preferences.
"""

    response = client.messages.create(
        model="claude-sonnet-4-5-20250929",
        max_tokens=4096,
        messages=[{"role": "user", "content": prompt}]
    )

    return response.content[0].text


def post_pr_comment(review_text, has_violations):
    """Post review as PR comment and set check status."""
    gh = Github(GITHUB_TOKEN)
    repo = gh.get_repo(REPO_NAME)
    pr = repo.get_pull(PR_NUMBER)

    # Post comment
    comment_body = f"""## 🤖 AI Architectural Review

{review_text}

---
<sub>Powered by Claude Sonnet 4.5 | [View principles](../blob/main/CLAUDE.md)</sub>
"""
    pr.create_issue_comment(comment_body)

    # Set check status
    if has_violations:
        print("❌ Architectural violations found - blocking merge")
        sys.exit(1)  # Fail workflow
    else:
        print("✅ Architectural review passed")
        sys.exit(0)


def main():
    base_ref = os.environ["BASE_REF"]
    head_ref = os.environ["HEAD_REF"]

    # Get PR details
    gh = Github(GITHUB_TOKEN)
    repo = gh.get_repo(REPO_NAME)
    pr = repo.get_pull(PR_NUMBER)
    pr_title = pr.title
    pr_description = pr.body

    # Check for skip flag
    if "[skip arch-review]" in pr_title.lower():
        print("⏭️  Skipping architectural review (found [skip arch-review] in PR title)")
        sys.exit(0)

    print(f"🔍 Reviewing PR #{PR_NUMBER}: {pr_title}")

    # Get changed files
    print("📂 Fetching changed files...")
    changed_files = get_changed_files(".", base_ref, head_ref)
    print(f"   Found {len(changed_files)} changed file(s)")

    # Read principles
    print("📖 Reading architectural principles from CLAUDE.md...")
    principles = read_claude_md()

    # Review with Claude
    print("🤖 Analyzing changes with Claude Sonnet 4.5...")
    review = review_changes(changed_files, principles, pr_title, pr_description)

    # Check for violations
    violations_section = review.split("### ❌ Violations")[1].split("###")[0] if "### ❌ Violations" in review else ""
    has_violations = violations_section.strip() not in ["", "None found.", "None found"]

    # Post to PR
    print("💬 Posting review comment to PR...")
    post_pr_comment(review, has_violations)


if __name__ == "__main__":
    main()
