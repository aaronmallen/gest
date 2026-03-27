#!/usr/bin/env bash
# Session-start hook — injected on startup, clear, and compact events.
# Outputs additional context that reminds the agent about the workflow.

set -euo pipefail

# Build the context message
read -r -d '' CONTEXT << 'CONTEXT_EOF' || true
## Workflow Reminder

Before responding, check if any of the following skills apply to the user's request:

### Workflow Skills
- `/brainstorm` — shape a rough idea into a product spec (stored as a gest artifact)
- `/plan` — analyze a spec and present scope assessment, create tasks
- `/implement <gest-id>` — implement a single issue
- `/orchestrate <gest-id>` — implement a multi-issue plan in parallel

### Free-Floating Skills
- `/code-review` — review current changes for correctness, style, and architecture
- `/format` — format and lint the project, audit against code style
- `/commit` — create a conventional commit following project conventions
- `/changelog` — generate or update the CHANGELOG.md from commits since latest release

### Core Principles
1. **Delegate VCS operations** to the **vcs-expert** agent — never run git/jj/git-butler commands directly
2. **Delegate task execution** to the **task-runner** agent — never run build/test/lint commands directly
3. **Human stays in the loop** — present options and ask, don't auto-decide
4. **Tests are the source of truth** — never modify existing tests unless explicitly asked
CONTEXT_EOF

# Output in Claude Code hook format
printf '{"hookSpecificOutput": {"additionalContext": "%s"}}' \
  "$(printf '%s' "$CONTEXT" | sed 's/\\/\\\\/g; s/"/\\"/g; s/$/\\n/g' | tr -d '\n')"
