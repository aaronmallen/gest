---
name: promote-task
description: "Promote a gest task to a GitHub Issue (e.g. /promote-task <id>)."
args: "<gest-id>"
---

# Promote Task

Promote a gest task to a GitHub Issue.

## Instructions

### 1. Read the Task

```sh
cargo run -- task show <id> --json
```

Extract:

- `title` — becomes the issue title
- `description` — becomes the issue body (see sanitization rules below)
- `tags` — mapped to GitHub labels using the vocabulary in `docs/process/labels.md`

### 2. Sanitize

**Sanitize the description before promoting.** The issue body must not contain internal gest references. Remove or
rewrite the following:

- **Gest IDs** — any gest short ID (e.g. `ktxolxqz`) or references like `gest task <id>` must be stripped. If a
  dependency or link references a gest entity, replace the ID with the entity's title.
- **Dependencies section** — if dependencies list only gest entities, remove the section entirely. If it mixes gest
  and external references, keep only the external ones.
- **File Structure / implementation details** — remove sections that describe internal file paths or implementation
  plans. The GitHub issue should describe *what* and *why*, not *how*.
- **Duplicate title** — do not repeat the title as an `# H1` heading in the body. GitHub already displays the title
  prominently; an H1 in the body is redundant.

### 3. Reshape to the Issue Template

Rewrite the sanitized content into the project's standard issue template. Use this exact section order and
headings — no extra sections, no "Proposed Solution" or "Problem Statement" headings, no duplicate title.

```markdown
## User Story
As a <role>, I want <capability> so that <benefit>.

## Context
<Background and motivation: why this is needed, relevant constraints, prior-art references.
Keep to 1–3 paragraphs. Technical framing is fine; implementation plans are not.>

## Acceptance Criteria
- [ ] <Measurable outcome 1>
- [ ] <Measurable outcome 2>
- [ ] `cargo build` and `cargo test` pass.

## Out of Scope
- <Explicit non-goal 1>
- <Explicit non-goal 2>
```

Template rules:

- **User Story is required.** Derive the role, capability, and benefit from the task description. If the source
  content doesn't state a user-facing benefit, infer one from context rather than omitting the section.
- **Context is required** and should explain *why*, not *how*. Pull motivation, constraints, and relevant prior-art
  pointers from the source. Do not include file:line references, variant names, or implementation plans unless a
  reader genuinely cannot understand the issue without them.
- **Acceptance Criteria is required** and uses GitHub task-list syntax (`- [ ]`). Each criterion should be
  independently verifiable. Always include a final `cargo build` / `cargo test` criterion unless the work is
  docs-only.
- **Out of Scope is required** when the source identifies non-goals; omit the section only if there are none.
- **No other top-level headings.** Do not add Summary, Problem, Proposed Solution, Breaking Change, References,
  Open Questions, or similar — fold any essential content from those sections into Context or Acceptance Criteria.
- **No code blocks showing implementation** (e.g., `impl X { ... }`). Prose descriptions only in Context.
- **Tables are allowed** in Context when they convey reference data (e.g., a code → name mapping).

For overarching/epic issues promoted from a spec or iteration, the same template applies. Use the `epic` type label
in addition to area labels.

### 4. Confirm and Create

Draft the `gh issue create` command and present it to the user for confirmation:

```sh
gh issue create \
  --title "<title>" \
  --label "<label1>,<label2>,..." \
  --body "<description>"
```

After the user confirms, execute the command. Extract the issue number from the output, then store it as task metadata:

```sh
cargo run -- task meta set <id> github-issue <number>
```

### 5. Report

Print a summary: the GitHub Issue number and URL.

Print: `the task has been promoted and linked via metadata`
