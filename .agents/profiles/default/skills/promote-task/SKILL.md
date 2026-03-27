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

**Sanitize the description before promoting.** The issue body must not contain internal gest
references. Remove or rewrite the following:

- **Gest IDs** — any gest short ID (e.g. `ktxolxqz`) or references like `gest task <id>` must be
  stripped. If a dependency or link references a gest entity, replace the ID with the entity's title.
- **Dependencies section** — if dependencies list only gest entities, remove the section entirely.
  If it mixes gest and external references, keep only the external ones.
- **File Structure / implementation details** — remove sections that describe internal file paths or
  implementation plans. The GitHub issue should describe *what* and *why*, not *how*.
- **Duplicate title** — do not repeat the title as an `# H1` heading in the body. GitHub already
  displays the title prominently; an H1 in the body is redundant.

### 3. Confirm and Create

Draft the `gh issue create` command and present it to the user for confirmation:

```sh
gh issue create \
  --title "<title>" \
  --label "<label1>,<label2>,..." \
  --body "<description>"
```

After the user confirms, execute the command. Extract the issue number from the output, then store
it as task metadata:

```sh
cargo run -- task meta set <id> github-issue <number>
```

### 4. Report

Print a summary: the GitHub Issue number and URL.

Print: `the task has been promoted and linked via metadata`
