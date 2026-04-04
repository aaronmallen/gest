---
name: write-spec
description: "Draft a product spec and store it as a gest artifact (e.g. /write-spec \"user authentication\")."
args: "<topic and approach>"
---

# Write Spec

Draft a product spec and store it as a gest artifact.

## Instructions

### 1. Draft the Spec

Write a spec using this structure. Omit any section that doesn't apply.

```markdown
# Spec: <Title>

## Problem Statement
What problem does this solve? Who is affected?

## Proposed Solution
How it works from the user's perspective. Focus on behavior, not implementation.

## Scope

### In Scope
- ...

### Out of Scope
- ...

## Acceptance Criteria
- [ ] Measurable outcome 1
- [ ] Measurable outcome 2

## Open Questions
- Anything still unresolved

## References
- Related specs, ADRs, issues
```

### 2. Review with User

Present the draft and iterate based on feedback. Keep it concise -- a spec should be short enough to read in a couple of
minutes.

### 3. Save

Pipe the spec content via stdin. When `--body` is omitted and stdin is piped, gest reads the body automatically. Use
`-q`
to get the bare artifact ID:

```sh
cat <<'EOF' | GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact create \
  --title "<title>" --type spec --tag "<area>,spec" -q
<spec content here>
EOF
```

Use bare tags (no `area:` or `type:` prefixes). Include the relevant area tag(s) from `docs/process/labels.md` and the
`spec` type tag.

The `-q` flag prints only the artifact ID, ready for downstream use.

### 4. Next Step

Print: `invoke /plan <id> when you're ready for the next step`
