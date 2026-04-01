---
name: gest
description: Gest CLI reference for managing artifacts, tasks, and iterations. Use when skills need to create, read, or link entities.
tools: Bash, Read
model: haiku
---

# Gest CLI Agent

Reference agent for interacting with gest -- the project's artifact, task, and iteration store. Gest stores data outside
the repository (XDG data directory). No `gest init` is needed; directories are created on first write.

All commands use `GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run --` as the invocation prefix.

## Entity Model

- **Artifacts** -- Markdown documents (specs, ADRs, RFCs). Stored by type.
- **Tasks** -- Actionable work items with status, links, priority, phase, and assignment.
- **Iterations** -- Execution plans that group tasks into phased, parallelizable work.

IDs are 8-character lowercase alphabetic strings. Prefix matching works -- you can use a shorter prefix if it's
unambiguous.

## Artifacts

### Create

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact create --title "<title>" --type <kind> --file <path>
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact create --title "<title>" --type <kind> --body "<inline body>"
```

Types: `spec`, `adr`, `rfc` (freeform string -- use these conventions).

Options: `--tags <comma-separated>`, `--metadata <key=value>`.

Output: `Created artifact <id>` -- extract the ID from this line.

### Show

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact show <id>          # human-readable
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact show --json <id>   # structured JSON
```

### List

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact list                     # active only
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact list --include-archived  # include archived
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact list --type <kind>       # filter by type
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact list --tag <tag>         # filter by tag
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact list --json              # structured JSON
```

### Update

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact update <id> --title "<new title>"
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact update <id> --body "<new body>"
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact update <id> --type <kind>
```

### Archive

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact archive <id>
```

### Tags

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact tag <id> <tag>
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact untag <id> <tag>
```

### Metadata

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact meta set <id> <key> <value>
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact meta get <id> <key>
```

## Tasks

### Create

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task create "<title>" --description "<desc>"
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task create "<title>" --status <status> --tags <tags>
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task create "<title>" --priority 1 --phase 2 --assigned-to agent-1
```

Options: `--description`, `--status`, `--tags <comma-separated>`, `--metadata <key=value>`, `--priority <0-4>`, `--phase
<number>`, `--assigned-to <actor>`.

Output: `Created task <id>` -- extract the ID from this line.

### Show

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task show <id>          # human-readable
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task show --json <id>   # structured JSON
```

### List

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task list                     # active only
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task list --all               # include resolved
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task list --status <status>   # filter by status
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task list --tag <tag>         # filter by tag
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task list --json              # structured JSON
```

### Update

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task update <id> --status <status>
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task update <id> --title "<new title>"
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task update <id> --priority 0
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task update <id> --phase 3
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task update <id> --assigned-to agent-2
```

Setting status to `done` or `cancelled` automatically archives the task. Setting an archived task's status to `open` or
`in-progress` automatically unarchives it.

### Task Fields

- `priority` -- P0-P4 where P0 is highest priority (optional)
- `phase` -- execution phase number for parallel grouping (optional). Tasks in the same phase are safe to run
  concurrently. Phases execute sequentially.
- `assigned_to` -- actor (human or agent) working on the task (optional)

### Links

Links connect tasks to other tasks or to artifacts.

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task link <task-id> <rel> <target-task-id>              # task-to-task
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task link <task-id> <rel> <target-artifact-id> --artifact  # task-to-artifact
```

Valid relation types:

- `blocks` -- this task blocks the target
- `blocked-by` -- this task is blocked by the target
- `relates-to` -- general association
- `child-of` -- this task is a child of the target (e.g., a task that implements a spec)
- `parent-of` -- the target is a child of this task

### Metadata

Freeform key-value pairs.

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task meta set <id> <key> <value>
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task meta get <id> <key>
```

### Notes

Notes are timestamped, attributed entries for recording decisions, progress updates, and observations.

```sh
# Add a note (human author from git config)
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task note add <task-id> --body "<text>"
# Add an agent-attributed note
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task note add <task-id> --agent <name> --body "<text>"
# List notes
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task note list <task-id>
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task note list <task-id> --json
# Show a single note
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task note show <task-id> <note-id>
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task note show <task-id> <note-id> --json
# Update a note
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task note update <task-id> <note-id> --body "<new text>"
# Delete a note
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task note delete <task-id> <note-id>
```

Author resolution: `--agent <name>` sets `author_type: agent`. Without `--agent`, author comes from
`git config user.name` / `user.email` with `author_type: human`. Notes appear in `task show` output
and `task show --json`.

### Tags

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task tag <id> <tag>
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task untag <id> <tag>
```

## Iterations

Iterations group tasks into an execution plan. They separate "how to execute" from the spec ("what to build").

### Create

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration create "<title>"
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration create "<title>" --description "<desc>" --tags <tags>
```

Output: `Created iteration <id>` -- extract the ID from this line.

### Show

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration show <id>          # human-readable
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration show --json <id>   # structured JSON
```

### List

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration list                 # active only
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration list --all           # include resolved
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration list --status <s>    # filter: active, completed, failed
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration list --json          # structured JSON
```

### Update

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration update <id> --status completed
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration update <id> --title "<new title>"
```

Setting status to `completed` or `failed` automatically resolves the iteration.

### Add / Remove Tasks

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration add <iteration-id> <task-id>      # add a task
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration remove <iteration-id> <task-id>   # remove a task
```

### Graph

Visualize the phased execution plan:

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration graph <id>          # jj-style tree output
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration graph --json <id>   # structured JSON
```

### Links

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration link <id> <rel> <target-id>              # to a task
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration link <id> <rel> <target-id> --artifact   # to an artifact
```

### Tags

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration tag <id> <tag>
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration untag <id> <tag>
```

### Metadata

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration meta set <id> <key> <value>
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration meta get <id> <key>
```

### Orchestration

These commands support multi-agent execution of iteration phases.

#### Status

Check iteration progress (active phase, task counts, assignees):

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration status <id>          # human-readable
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration status <id> --json   # structured JSON
```

JSON output includes: `active_phase`, `total_phases`, `phase_progress` (`done`/`total`), `blocked`, `in_progress`,
`assignees`, `overall_progress` (`done`/`total`).

#### Next

Find and optionally claim the next available task in the iteration:

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration next <id>                          # show next task
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration next <id> --claim --agent <name>   # atomically claim it
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration next <id> --json                   # structured JSON
```

Exits with code 2 when no tasks remain (distinguishes "idle" from "error").

Options: `--claim` (set task to in-progress), `--agent <name>` (set assigned_to).

#### Advance

Move to the next phase once the current phase is complete:

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration advance <id>           # advance if phase is done
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration advance <id> --force   # advance past stuck tasks
```

## Undo

Reverse the most recent mutating command(s) by restoring file snapshots. Every mutating CLI command is automatically
recorded in a local event store. Non-mutating commands (show, list, search) are not recorded.

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- undo       # undo last command
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- undo 3     # undo last 3 commands
```

Undo supports create (deletes file), modify (restores prior content), and delete (recreates file) operations. The undo
command itself is not recorded, so repeated calls walk backwards through history.

## Search

```sh
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- search "<query>"                     # search all entities
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- search --json "<query>"              # structured JSON
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- search --include-archived "<query>"  # include archived
```

## ID Extraction

When creating entities, the output format is:

```text
Created artifact <id>
Created task <id>
Created iteration <id>
```

Extract the last word from the output line to get the ID. This ID is used to reference the entity in subsequent commands
(show, update, link, etc.).

## Workflow Patterns

### Creating an iteration from a spec

```sh
# Create the iteration
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration create "Sprint: Feature X"
# Link it to the source spec
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration link <iteration-id> child-of <spec-id> --artifact
# Create tasks with phase and priority
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task create "Add parser types" --phase 1 --priority 1
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task create "Add CLI flag" --phase 1 --priority 2
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task create "Integrate parser" --phase 2 --priority 0
# Link tasks to spec and add to iteration
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task link <task-id> child-of <spec-id> --artifact
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration add <iteration-id> <task-id>
# Set blocking dependencies
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task link <task-c> blocked-by <task-a>
# View the execution graph
GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration graph <iteration-id>
```

### Finding tasks linked to a spec

Use `GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task list --json` and inspect the `links` array
for entries referencing the artifact ID, or use
`GEST_DATA_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- search "<spec title>"` to find related entities.
