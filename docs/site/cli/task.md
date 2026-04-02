# gest task

Create, update, list, and manage tasks. Tasks represent units of work with a title,
description, status, priority, tags, metadata, and relationship links.

## Usage

```text
gest task <COMMAND> [OPTIONS]
```

## Subcommands

| Command                      | Aliases | Description                                                         |
|------------------------------|---------|---------------------------------------------------------------------|
| [`block`](#task-block)       |         | Shortcut for `task link <id> blocks <target>`                       |
| [`cancel`](#task-cancel)     |         | Cancel a task (shortcut for `task update <id> --status cancelled`)  |
| [`complete`](#task-complete) |         | Mark a task as done (shortcut for `task update <id> --status done`) |
| [`create`](#task-create)     | `new`   | Create a new task                                                   |
| [`list`](#task-list)         | `ls`    | List tasks with optional filters                                    |
| [`show`](#task-show)         | `view`  | Display a task's full details                                       |
| [`update`](#task-update)     | `edit`  | Update a task's fields                                              |
| [`tag`](#task-tag)           |         | Add tags to a task                                                  |
| [`untag`](#task-untag)       |         | Remove tags from a task                                             |
| [`link`](#task-link)         |         | Create a relationship between entities                              |
| [`meta`](#task-meta)         |         | Read or write metadata fields                                       |
| [`note`](#task-note)         |         | Manage notes on a task                                              |

---

## task block

Shortcut for `task link <id> blocks <target>`. Creates a blocking relationship between two
tasks (or a task and an artifact).

```text
gest task block [OPTIONS] <ID> <BLOCKING_ID>
```

### Arguments

| Argument        | Description                                                          |
|-----------------|----------------------------------------------------------------------|
| `<ID>`          | Source task ID or unique prefix (the task that blocks)               |
| `<BLOCKING_ID>` | Target task or artifact ID or unique prefix (the task being blocked) |

### Options

| Flag           | Description                                                             |
|----------------|-------------------------------------------------------------------------|
| `--artifact`   | Target is an artifact instead of a task (no reciprocal link is created) |
| `-j, --json`   | Output the task as JSON after linking                                   |
| `-q, --quiet`  | Output only the task ID                                                 |

### Examples

```sh
# Task abc123 blocks task def456
gest task block abc123 def456

# Task blocks an artifact
gest task block abc123 art789 --artifact
```

---

## task cancel

Cancel a task. Shortcut for `task update <id> --status cancelled`.

```text
gest task cancel [OPTIONS] <ID>
```

### Arguments

| Argument | Description              |
|----------|--------------------------|
| `<ID>`   | Task ID or unique prefix |

### Options

| Flag          | Description            |
|---------------|------------------------|
| `-j, --json`  | Output as JSON         |
| `-q, --quiet` | Print only the task ID |

### Examples

```sh
gest task cancel abc123
```

---

## task complete

Mark a task as done. Shortcut for `task update <id> --status done`.

```text
gest task complete [OPTIONS] <ID>
```

### Arguments

| Argument | Description              |
|----------|--------------------------|
| `<ID>`   | Task ID or unique prefix |

### Options

| Flag          | Description            |
|---------------|------------------------|
| `-j, --json`  | Output as JSON         |
| `-q, --quiet` | Print only the task ID |

### Examples

```sh
gest task complete abc123
```

---

## task create

Create a new task with optional metadata, tags, and status.

When `--description` is omitted and stdin is a terminal, `$EDITOR` opens for interactive
editing. When stdin is a pipe, the piped content is used as the description body.

```text
gest task create [OPTIONS] [TITLE]
```

### Arguments

| Argument  | Description |
|-----------|-------------|
| `[TITLE]` | Task title  |

### Options

| Flag                              | Description                                                                     |
|-----------------------------------|---------------------------------------------------------------------------------|
| `--assigned-to <ASSIGNED_TO>`     | Actor assigned to this task                                                     |
| `--batch`                         | Read NDJSON from stdin (one task per line)                                      |
| `-d, --description <DESCRIPTION>` | Description text (opens `$EDITOR` if omitted and stdin is a terminal)           |
| `-i, --iteration <ITERATION>`     | Add the task to an iteration (ID or prefix)                                     |
| `-j, --json`                      | Output the created task as JSON                                                 |
| `-l, --link <LINK>`               | Create a link on the new task (repeatable, format: `<rel>:<target_id>`)         |
| `-m, --metadata <METADATA>`       | Key=value metadata pair (repeatable, e.g. `-m key=value`)                       |
| `--phase <PHASE>`                 | Execution phase for parallel grouping                                           |
| `-p, --priority <PRIORITY>`       | Priority level (0-4, where 0 is highest)                                        |
| `-q, --quiet`                     | Print only the task ID                                                          |
| `-s, --status <STATUS>`           | Initial status: `open`, `in-progress`, `done`, or `cancelled` (default: `open`) |
| `--tag <TAG>`                     | Tag (repeatable, or comma-separated)                                            |

### Examples

```sh
# Create a simple task
gest task create "Implement login page"

# Create a task with description and tags
gest task create "Fix memory leak" -d "OOM after 24h uptime" --tag "bug,critical"

# Create a high-priority task assigned to an agent
gest task create "Write migration" -p 0 --assigned-to agent --phase 1

# Create a task and add it to an iteration with a link
gest task create "Add auth" -i iter123 -l child-of:spec456

# Pipe description from stdin
echo "Detailed description here" | gest task create "My task"

# Batch-create tasks from NDJSON
cat tasks.ndjson | gest task create --batch

# Machine-readable output
gest task create "Quick task" --json
gest task create "Quick task" -q
```

---

## task list

List tasks, optionally filtered by status or tag.

```text
gest task list [OPTIONS]
```

### Options

| Flag                             | Description                                                     |
|----------------------------------|-----------------------------------------------------------------|
| `-a, --all`                      | Include resolved (done/cancelled) tasks                         |
| `--assigned-to <ASSIGNED_TO>`    | Filter by assigned-to name                                      |
| `-j, --json`                     | Output task list as JSON                                        |
| `-s, --status <STATUS>`          | Filter by status: `open`, `in-progress`, `done`, or `cancelled` |
| `--tag <TAG>`                    | Filter by tag                                                   |

### Examples

```sh
# List active tasks
gest task list

# List all tasks including resolved
gest task list --all

# Filter by status
gest task list -s in-progress

# Filter by assignee
gest task list --assigned-to agent

# JSON output for scripting
gest task list --json
```

---

## task show

Display a task's full details, description, and links.

```text
gest task show [OPTIONS] <ID>
```

### Arguments

| Argument | Description              |
|----------|--------------------------|
| `<ID>`   | Task ID or unique prefix |

### Options

| Flag         | Description                 |
|--------------|-----------------------------|
| `-j, --json` | Output task details as JSON |

### Examples

```sh
# Show task by full ID
gest task show abc123

# Show task by prefix
gest task show ab

# JSON output
gest task show abc123 --json
```

---

## task update

Update a task's title, description, status, tags, or metadata.

```text
gest task update [OPTIONS] <ID>
```

### Arguments

| Argument | Description              |
|----------|--------------------------|
| `<ID>`   | Task ID or unique prefix |

### Options

| Flag                              | Description                                                             |
|-----------------------------------|-------------------------------------------------------------------------|
| `--assigned-to <ASSIGNED_TO>`     | Actor assigned to this task                                             |
| `-d, --description <DESCRIPTION>` | New description text                                                    |
| `-j, --json`                      | Output as JSON                                                          |
| `-m, --metadata <METADATA>`       | Key=value metadata pair, merged with existing (repeatable)              |
| `--phase <PHASE>`                 | Execution phase for parallel grouping                                   |
| `-p, --priority <PRIORITY>`       | Priority level (0-4, where 0 is highest)                                |
| `-q, --quiet`                     | Print only the task ID                                                  |
| `-s, --status <STATUS>`           | New status (done/cancelled auto-resolves; open/in-progress un-resolves) |
| `--tag <TAG>`                     | Replace all tags (repeatable, or comma-separated)                       |
| `-t, --title <TITLE>`             | New title                                                               |

### Examples

```sh
# Mark a task as done
gest task update abc123 -s done

# Update title and description
gest task update abc123 -t "New title" -d "Updated description"

# Add metadata
gest task update abc123 -m estimate=3h -m complexity=high

# Machine-readable output
gest task update abc123 -s done --json
```

---

## task tag

Add tags to a task, deduplicating with any existing tags.

```text
gest task tag [OPTIONS] <ID> [TAGS]...
```

### Arguments

| Argument    | Description                              |
|-------------|------------------------------------------|
| `<ID>`      | Task ID or unique prefix                 |
| `[TAGS]...` | Tags to add (space or comma-separated)   |

### Options

| Flag          | Description                           |
|---------------|---------------------------------------|
| `-j, --json`  | Output the task as JSON after tagging |
| `-q, --quiet` | Output only the task ID               |

### Examples

```sh
gest task tag abc123 bug critical
gest task tag abc123 bug,critical
```

---

## task untag

Remove tags from a task.

```text
gest task untag [OPTIONS] <ID> [TAGS]...
```

### Arguments

| Argument    | Description                                |
|-------------|--------------------------------------------|
| `<ID>`      | Task ID or unique prefix                   |
| `[TAGS]...` | Tags to remove (space or comma-separated)  |

### Options

| Flag          | Description                             |
|---------------|-----------------------------------------|
| `-j, --json`  | Output the task as JSON after untagging |
| `-q, --quiet` | Output only the task ID                 |

### Examples

```sh
gest task untag abc123 critical
```

---

## task link

Create a relationship between a task and another task or artifact.

```text
gest task link [OPTIONS] <ID> <REL> <TARGET_ID>
```

### Arguments

| Argument      | Description                                                                      |
|---------------|----------------------------------------------------------------------------------|
| `<ID>`        | Source task ID or unique prefix                                                  |
| `<REL>`       | Relationship type: `blocked-by`, `blocks`, `child-of`, `parent-of`, `relates-to` |
| `<TARGET_ID>` | Target task or artifact ID or unique prefix                                      |

### Options

| Flag          | Description                                                             |
|---------------|-------------------------------------------------------------------------|
| `--artifact`  | Target is an artifact instead of a task (no reciprocal link is created) |
| `-j, --json`  | Output the task as JSON after linking                                   |
| `-q, --quiet` | Output only the task ID                                                 |

### Examples

```sh
# Task blocks another task
gest task link abc123 blocks def456

# Task relates to an artifact
gest task link abc123 relates-to art789 --artifact
```

---

## task meta

Read or write task metadata fields. Metadata uses dot-delimited key paths for nested values.

```text
gest task meta <COMMAND>
```

### meta get

Retrieve a single metadata value.

```text
gest task meta get [OPTIONS] <ID> <PATH>
```

| Argument | Description                                 |
|----------|---------------------------------------------|
| `<ID>`   | Task ID or unique prefix                    |
| `<PATH>` | Dot-delimited key path (e.g. `outer.inner`) |

| Flag     | Description                           |
|----------|---------------------------------------|
| `--json` | Output as a JSON object               |
| `--raw`  | Output the bare value with no styling |

### meta set

Set a metadata value. Strings, numbers, and booleans are auto-detected.

```text
gest task meta set [OPTIONS] <ID> <PATH> <VALUE>
```

| Argument  | Description                                 |
|-----------|---------------------------------------------|
| `<ID>`    | Task ID or unique prefix                    |
| `<PATH>`  | Dot-delimited key path (e.g. `outer.inner`) |
| `<VALUE>` | Value to set                                |

| Flag          | Description              |
|---------------|--------------------------|
| `-j, --json`  | Output as JSON           |
| `-q, --quiet` | Print only the entity ID |

### Examples

```sh
# Set a metadata field
gest task meta set abc123 estimate "3 hours"

# Read it back
gest task meta get abc123 estimate

# JSON output
gest task meta get abc123 estimate --json

# Raw value (no styling)
gest task meta get abc123 estimate --raw
```

---

## task note

Manage notes on a task. Notes are timestamped, attributed entries for recording decisions,
progress updates, and observations — analogous to comments on a GitHub issue.

```text
gest task note <COMMAND>
```

| Command                         | Aliases | Description              |
|---------------------------------|---------|--------------------------|
| [`add`](#note-add)              |         | Add a note to a task     |
| [`list`](#note-list)            | `ls`    | List all notes on a task |
| [`show`](#note-show)            | `view`  | Show a single note       |
| [`update`](#note-update)        |         | Update a note's body     |
| [`delete`](#note-delete)        |         | Delete a note            |

### note add

Add a note to a task. Author defaults to `git config user.name` / `user.email`.

When `--body` is omitted and stdin is a terminal, `$EDITOR` opens for interactive editing.
When stdin is a pipe, the piped content is used as the note body.

```text
gest task note add [OPTIONS] <ID>
```

| Argument | Description              |
|----------|--------------------------|
| `<ID>`   | Task ID or unique prefix |

| Flag                | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| `--agent <AGENT>`   | Agent name for attribution (mutually exclusive with git-derived authorship) |
| `-b, --body <BODY>` | Note body text (opens `$EDITOR` if omitted and stdin is a terminal)         |
| `-j, --json`        | Output as JSON                                                              |
| `-q, --quiet`       | Print only the note ID                                                      |

### note list

List all notes on a task.

```text
gest task note list [OPTIONS] <ID>
```

| Argument | Description              |
|----------|--------------------------|
| `<ID>`   | Task ID or unique prefix |

| Flag         | Description    |
|--------------|----------------|
| `-j, --json` | Output as JSON |

### note show

Show a single note with full attribution and rendered markdown body.

```text
gest task note show [OPTIONS] <TASK_ID> <NOTE_ID>
```

| Argument    | Description              |
|-------------|--------------------------|
| `<TASK_ID>` | Task ID or unique prefix |
| `<NOTE_ID>` | Note ID or unique prefix |

| Flag         | Description    |
|--------------|----------------|
| `-j, --json` | Output as JSON |

### note update

Update a note's body.

```text
gest task note update [OPTIONS] <TASK_ID> <NOTE_ID>
```

| Argument    | Description              |
|-------------|--------------------------|
| `<TASK_ID>` | Task ID or unique prefix |
| `<NOTE_ID>` | Note ID or unique prefix |

| Flag                | Description                                                                   |
|---------------------|-------------------------------------------------------------------------------|
| `-b, --body <BODY>` | New body text (opens `$EDITOR` pre-filled if omitted and stdin is a terminal) |
| `-j, --json`        | Output as JSON                                                                |
| `-q, --quiet`       | Print only the note ID                                                        |

### note delete

Delete a note from a task.

```text
gest task note delete <TASK_ID> <NOTE_ID>
```

| Argument    | Description              |
|-------------|--------------------------|
| `<TASK_ID>` | Task ID or unique prefix |
| `<NOTE_ID>` | Note ID or unique prefix |

### Examples

```sh
# Add a human note (author from git config)
gest task note add abc123 --body "Found the root cause in the parser"

# Add an agent note
gest task note add abc123 --agent claude --body "Completed code review, no issues found"

# Pipe note body from stdin
echo "Investigation notes" | gest task note add abc123

# List notes
gest task note list abc123

# Show a specific note
gest task note show abc123 nfkbqmrx

# Update a note
gest task note update abc123 nfkbqmrx --body "Updated analysis"

# Delete a note
gest task note delete abc123 nfkbqmrx
```
