---
name: gest
description: Gest CLI reference for managing artifacts and tasks. Use when skills need to create, read, or link entities.
tools: Bash, Read
model: haiku
---

# Gest CLI Agent

Reference agent for interacting with gest -- the project's artifact and task store. Gest stores data
outside the repository (XDG data directory). No `gest init` is needed; directories are created on
first write.

All commands use `cargo run --` as the invocation prefix.

## Entity Model

- **Artifacts** -- Markdown documents (specs, ADRs, RFCs). Stored by type.
- **Tasks** -- Actionable work items with status, links, and freeform metadata. Used for issues and
  plan items.

IDs are 8-character lowercase alphabetic strings. Prefix matching works -- you can use a shorter
prefix if it's unambiguous.

## Artifacts

### Create

```sh
cargo run -- artifact create --title "<title>" --type <kind> --file <path>
cargo run -- artifact create --title "<title>" --type <kind> --body "<inline body>"
```

Types: `spec`, `adr`, `rfc` (freeform string -- use these conventions).

Options: `--tags <comma-separated>`, `--metadata <key=value>`.

Output: `Created artifact <id>` -- extract the ID from this line.

### Show

```sh
cargo run -- artifact show <id>          # human-readable
cargo run -- artifact show --json <id>   # structured JSON
```

### List

```sh
cargo run -- artifact list                     # active only
cargo run -- artifact list --include-archived  # include archived
cargo run -- artifact list --type <kind>       # filter by type
cargo run -- artifact list --tag <tag>         # filter by tag
cargo run -- artifact list --json              # structured JSON
```

### Update

```sh
cargo run -- artifact update <id> --title "<new title>"
cargo run -- artifact update <id> --body "<new body>"
cargo run -- artifact update <id> --type <kind>
```

### Archive

```sh
cargo run -- artifact archive <id>
```

### Tags

```sh
cargo run -- artifact tag <id> <tag>
cargo run -- artifact untag <id> <tag>
```

### Metadata

```sh
cargo run -- artifact meta set <id> <key> <value>
cargo run -- artifact meta get <id> <key>
```

## Tasks

### Create

```sh
cargo run -- task create "<title>" --description "<desc>"
cargo run -- task create "<title>" --status <status> --tags <tags>
```

Options: `--description`, `--status`, `--tags <comma-separated>`, `--metadata <key=value>`,
`--link <rel>:<target_id>`.

Output: `Created task <id>` -- extract the ID from this line.

### Show

```sh
cargo run -- task show <id>          # human-readable
cargo run -- task show --json <id>   # structured JSON
```

### List

```sh
cargo run -- task list                     # active only
cargo run -- task list --include-archived  # include archived
cargo run -- task list --status <status>   # filter by status
cargo run -- task list --tag <tag>         # filter by tag
cargo run -- task list --json              # structured JSON
```

### Update

```sh
cargo run -- task update <id> --status <status>
cargo run -- task update <id> --title "<new title>"
cargo run -- task update <id> --description "<new desc>"
```

Setting status to `done` or `cancelled` automatically archives the task. Setting an archived task's
status to `open` or `in-progress` automatically unarchives it.

### Links

Links connect tasks to other tasks or to artifacts.

```sh
cargo run -- task link <task-id> <rel> <target-task-id>              # task-to-task
cargo run -- task link <task-id> <rel> <target-artifact-id> --artifact  # task-to-artifact
```

Valid relation types:

- `blocks` -- this task blocks the target
- `blocked-by` -- this task is blocked by the target
- `relates-to` -- general association
- `child-of` -- this task is a child of the target (e.g., a task that implements a spec)
- `parent-of` -- the target is a child of this task

### Metadata

Freeform key-value pairs for orchestration data.

```sh
cargo run -- task meta set <id> <key> <value>
cargo run -- task meta get <id> <key>
```

Conventions for orchestration metadata:

- `wave` -- execution wave number (e.g., `1`, `2`)
- `parallel` -- whether this task can run in parallel with others in the same wave (`true`/`false`)
- `complexity` -- estimated complexity (`small`, `medium`, `large`)

### Tags

```sh
cargo run -- task tag <id> <tag>
cargo run -- task untag <id> <tag>
```

## Search

```sh
cargo run -- search "<query>"                     # search all entities
cargo run -- search --json "<query>"              # structured JSON
cargo run -- search --include-archived "<query>"  # include archived
```

## ID Extraction

When creating entities, the output format is:

```text
Created artifact <id>
Created task <id>
```

Extract the last word from the output line to get the ID. This ID is used to reference the entity
in subsequent commands (show, update, link, etc.).

## Workflow Patterns

### Creating a spec artifact from a file

```sh
# Write content to a temp file, then import
cargo run -- artifact create --title "My Spec" --type spec --file /tmp/my-spec.md
```

### Creating a task linked to a spec

```sh
# Create the task
cargo run -- task create "Implement feature X" --description "..." --status open
# Link it to the source spec artifact
cargo run -- task link <task-id> child-of <artifact-id> --artifact
# Set orchestration metadata
cargo run -- task meta set <task-id> wave 1
cargo run -- task meta set <task-id> parallel true
```

### Finding tasks linked to a spec

Use `cargo run -- task list --json` and inspect the `links` array for entries referencing the
artifact ID, or use `cargo run -- search "<spec title>"` to find related entities.
