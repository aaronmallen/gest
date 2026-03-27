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

Present the draft and iterate based on feedback. Keep it concise -- a spec should be short enough to
read in a couple of minutes.

### 3. Save

Create a gest artifact with the spec content inline:

```sh
cargo run -- artifact create --title "<title>" --type spec --body "<content>"
```

Extract the artifact ID from the output.

### 4. Next Step

Print: `invoke /plan <id> when you're ready for the next step`
