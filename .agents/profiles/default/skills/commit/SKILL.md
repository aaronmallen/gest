---
name: commit
description: Create a conventional commit for the current changes.
---

# Commit

Create a commit following project conventions.

## Instructions

### 1. Review Changes

Delegate to the **vcs-expert** agent to:

- Show `jj status` (what files are changed)
- Show `jj diff` (what the changes are)
- Show `jj log` for recent commits (to match style)

### 2. Draft Commit Message

Follow the conventions in `docs/dev/commits.md`:

```text
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

**Types:** feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert

**Rules:**

- If a scope can reasonably apply, it **must** be included
- Multiple scopes are comma-separated
- Imperative mood ("add feature" not "added feature")
- First line under 72 characters
- Append `!` after type/scope for breaking changes
- **Never** reference gest task or artifact IDs in commit messages

**GitHub Issue references:**

If the current work relates to a gest task, check whether the task has `github-issue` metadata:

```sh
cargo run -- task show <id> --json
```

If the `metadata` object contains a `github-issue` key, include a footer referencing that issue
(e.g. `Closes #42`). If there is no `github-issue` metadata, do not add an issue reference.

### 3. Confirm

Present the draft commit message to the user for approval. Do not commit without confirmation.

### 4. Execute

Delegate to the **vcs-expert** agent to create the commit with the approved message.
