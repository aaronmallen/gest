#!/usr/bin/env bash
# Session-start hook — injects git-specific VCS context.

set -euo pipefail

read -r -d '' CONTEXT << 'CONTEXT_EOF' || true
## VCS Context

This project is configured to use **git** for version control. Always delegate VCS operations to the
**vcs-expert** agent, which uses standard `git` commands. Do not use `jj` or `git-butler` commands.
CONTEXT_EOF

printf '{"hookSpecificOutput": {"additionalContext": "%s"}}' \
  "$(printf '%s' "$CONTEXT" | sed 's/\\/\\\\/g; s/"/\\"/g; s/$/\\n/g' | tr -d '\n')"
