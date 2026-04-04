---
name: write-issue
description: "Draft an issue and store it as a gest task (e.g. /write-issue \"add CLI flag parsing\")."
args: "<issue topic and context>"
---

# Write Issue

Draft an issue and store it as a gest task.

## Instructions

### 1. Draft the Issue

Use this structure. Omit any section that is not relevant — only User Story and Acceptance Criteria are required.

```markdown
# <Title>

## User Story
As a <role>, I want <goal> so that <benefit>.

## Acceptance Criteria
- <measurable criterion>
- <measurable criterion>

## Dependencies
- `<gest-id>` — <why needed>, or "None"

## File Structure
- `<path>` — <what goes here>
```

### 2. Save

Pipe the issue content via stdin as the description. When `-d` is omitted and stdin is piped, gest reads the description
automatically. Use `-q` to get the bare task ID. Apply tags for type, area, and priority using the vocabulary from
`docs/process/labels.md`. Use bare tags -- no namespace prefixes like `area:` or `type:`:

```sh
cat <<'EOF' | GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task create "<title>" \
  --tag "enhancement,cli,p2" -q
<issue content here>
EOF
```

Tag examples: `bug`, `enhancement`, `chore`, `cli`, `model`, `storage`, `server`, `ui`, `config`, `docs`, `p0`-`p4`.

Inline flags available at creation time (use when the caller provides context):

- `--phase <n>` -- execution phase for parallel grouping
- `-p <0-4>` -- priority level (0 is highest)
- `-l <rel>:<target_id>` -- create a link (repeatable, e.g. `-l child-of:abcd1234 -l blocked-by:efgh5678`)
- `-i <iteration-id>` -- add the task to an iteration

The `-q` flag prints only the task ID, ready for downstream use.

### 3. Next Step

Print: `invoke /implement <id> when you're ready for the next step`
