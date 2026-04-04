---
name: write-adr
description: "Draft an Architecture Decision Record and store it as a gest artifact (e.g. /write-adr \"storage backend choice\")."
args: "<decision topic>"
---

# Write ADR

Draft an Architecture Decision Record. ADRs are stored as gest artifacts during drafting and committed to `docs/design/`
during implementation.

## Instructions

### 1. Determine the ID

Check existing ADRs in `docs/design/` to determine the next sequential ID. During initial drafting, use `id: draft` and
`# ADR-DRAFT: Title`.

### 2. Draft the ADR

Use the template from `docs/process/writing-adrs.md`. Key sections:

- **Summary** -- one paragraph explaining the decision
- **Context** -- why is this decision needed?
- **Decision** -- what we're going to do
- **Consequences** -- positive and negative effects

Omit any section or frontmatter field that doesn't apply. Do not include empty sections.

### 3. Review with User

Present the draft and iterate. ADRs should be clear enough that a future contributor can understand the decision without
additional context.

### 4. Save

Pipe the ADR content via stdin. When `--body` is omitted and stdin is piped, gest reads the body automatically. Use `-q`
to get the bare artifact ID:

```sh
cat <<'EOF' | GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact create \
  --title "<title>" --type adr --tag "<area>,adr" -q
<adr content here>
EOF
```

Use bare tags (no `area:` or `type:` prefixes). Include the relevant area tag(s) from `docs/process/labels.md` and the
`adr` type tag.

The `-q` flag prints only the artifact ID, ready for downstream use.

### 5. Next Step

Print: `invoke /implement <id> when you're ready for the next step`
