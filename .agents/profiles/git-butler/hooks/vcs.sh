#!/usr/bin/env bash
# Session-start hook — injects git-butler-specific VCS context.

set -euo pipefail

read -r -d '' CONTEXT << 'CONTEXT_EOF' || true
## VCS Context

This project is configured to use **Git Butler** for version control. Always delegate VCS operations
to the **vcs-expert** agent, which uses `git-butler` commands for write operations and read-only `git`
commands for inspection. Do not use `git commit`, `git checkout`, `git switch`, or `git branch`
directly — these can desync Git Butler's state.
CONTEXT_EOF

printf '{"hookSpecificOutput": {"additionalContext": "%s"}}' \
  "$(printf '%s' "$CONTEXT" | sed 's/\\/\\\\/g; s/"/\\"/g; s/$/\\n/g' | tr -d '\n')"
