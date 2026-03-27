---
name: write-issue
description: "Draft an issue and store it as a gest task (e.g. /write-issue \"add CLI flag parsing\")."
args: "<issue topic and context>"
---

# Write Issue

Draft an issue and store it as a gest task.

## Instructions

### 1. Draft the Issue

Use this structure. Omit any section that is not relevant — only User Story and Acceptance Criteria
are required.

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

Create a gest task with the issue content. Apply tags for type, area, and priority using the
vocabulary from `docs/process/labels.md`:

```sh
cargo run -- task create "<title>" --description "<content>" --tags "<type>,<area>,<priority>"
```

Extract the task ID from the output.

### 3. Next Step

Print: `invoke /implement <id> when you're ready for the next step`
