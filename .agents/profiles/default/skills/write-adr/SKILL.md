---
name: write-adr
description: "Draft an Architecture Decision Record and store it as a gest artifact (e.g. /write-adr \"storage backend choice\")."
args: "<decision topic>"
---

# Write ADR

Draft an Architecture Decision Record. ADRs are stored as gest artifacts during drafting and
committed to `docs/design/` during implementation.

## Instructions

### 1. Determine the ID

Check existing ADRs in `docs/design/` to determine the next sequential ID. During initial drafting,
use `id: draft` and `# ADR-DRAFT: Title`.

### 2. Draft the ADR

Use the template from `docs/process/writing-adrs.md`. Key sections:

- **Summary** -- one paragraph explaining the decision
- **Context** -- why is this decision needed?
- **Decision** -- what we're going to do
- **Consequences** -- positive and negative effects

Omit any section or frontmatter field that doesn't apply. Do not include empty sections.

### 3. Review with User

Present the draft and iterate. ADRs should be clear enough that a future contributor can understand
the decision without additional context.

### 4. Save

Create a gest artifact with the ADR content inline:

```sh
cargo run -- artifact create --title "<title>" --type adr --body "<content>"
```

Extract the artifact ID from the output.

### 5. Next Step

Print: `invoke /implement <id> when you're ready for the next step`
