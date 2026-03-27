#!/usr/bin/env bash
# PreToolUse hook — injects code-style and testing conventions before Write/Edit tool calls.
# Only triggers for source code files, not documentation or config.

set -euo pipefail

# Read the tool input from stdin
INPUT="$(cat)"

# Extract the file path being edited
FILE_PATH="$(printf '%s' "$INPUT" | grep -o '"file_path"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*"file_path"[[:space:]]*:[[:space:]]*"//; s/"$//')"

# Skip injection for non-source files (docs, config, tmp, markdown, toml, yaml, json)
case "$FILE_PATH" in
  */docs/*|*/tmp/*|*.md|*.toml|*.yaml|*.yml|*.json|*.lock|*.gitignore|*.editorconfig)
    exit 0
    ;;
esac

PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
CONTEXT=""

# Inject code style if it exists
if [ -f "$PROJECT_ROOT/docs/dev/code-style.md" ]; then
  CONTEXT="$(cat "$PROJECT_ROOT/docs/dev/code-style.md")"
fi

# Inject testing conventions if it exists
if [ -f "$PROJECT_ROOT/docs/dev/testing.md" ]; then
  if [ -n "$CONTEXT" ]; then
    CONTEXT="$CONTEXT

---

"
  fi
  CONTEXT="$CONTEXT$(cat "$PROJECT_ROOT/docs/dev/testing.md")"
fi

# Nothing to inject
if [ -z "$CONTEXT" ]; then
  exit 0
fi

# Output in Claude Code hook format
printf '{"hookSpecificOutput": {"additionalContext": "%s"}}' \
  "$(printf '%s' "$CONTEXT" | sed 's/\\/\\\\/g; s/"/\\"/g; s/$/\\n/g' | tr -d '\n')"
