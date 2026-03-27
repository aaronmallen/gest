#!/usr/bin/env bash
# PreToolUse hook — injects commit conventions before commit commands.
# Triggers for both git and jj commit commands.

set -euo pipefail

# Read the tool input from stdin
INPUT="$(cat)"

# Extract the command being run
COMMAND="$(printf '%s' "$INPUT" | grep -o '"command"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*"command"[[:space:]]*:[[:space:]]*"//; s/"$//')"

# Only trigger for commit commands (git, jj, or git-butler)
case "$COMMAND" in
  *git\ commit*|*git\ -c\ *commit*|*jj\ commit*|*jj\ describe*|*git-butler\ branch\ commit*)
    ;;
  *)
    exit 0
    ;;
esac

PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

if [ ! -f "$PROJECT_ROOT/docs/dev/commits.md" ]; then
  exit 0
fi

CONTEXT="$(cat "$PROJECT_ROOT/docs/dev/commits.md")"

printf '{"hookSpecificOutput": {"additionalContext": "%s"}}' \
  "$(printf '%s' "$CONTEXT" | sed 's/\\/\\\\/g; s/"/\\"/g; s/$/\\n/g' | tr -d '\n')"
