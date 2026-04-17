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
| [`claim`](#task-claim)       |         | Claim a task (assign and mark in-progress)                          |
| [`complete`](#task-complete) |         | Mark a task as done (shortcut for `task update <id> --status done`) |
| [`create`](#task-create)     | `new`   | Create a new task                                                   |
| [`delete`](#task-delete)     | `rm`    | Delete a task and its dependent rows                                |
| [`list`](#task-list)         | `ls`    | List tasks with optional filters                                    |
| [`show`](#task-show)         | `view`  | Display a task's full details                                       |
| [`update`](#task-update)     | `edit`  | Update a task's fields                                              |
| [`tag`](#task-tag)           |         | Add tags to a task                                                  |
| [`untag`](#task-untag)       |         | Remove tags from a task                                             |
| [`link`](#task-link)         |         | Create a relationship between entities                              |
| [`unlink`](#task-unlink)     |         | Remove a relationship between entities                              |
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

## task claim

Claim a task by assigning it to an author and marking it `in-progress`. By default the
author is derived from `git config user.name`; use `--as` to override.

```text
gest task claim [OPTIONS] <ID>
```

### Arguments

| Argument | Description              |
|----------|--------------------------|
| `<ID>`   | Task ID or unique prefix |

### Options

| Flag               | Description                                              |
|--------------------|----------------------------------------------------------|
| `--as <AS_AUTHOR>` | Claim as a specific author name (defaults to git user)   |
| `-j, --json`       | Output as JSON                                           |
| `-q, --quiet`      | Print only the task ID                                   |

### Examples

```sh
# Claim as the current git user
gest task claim abc123

# Claim on behalf of an agent
gest task claim abc123 --as implement-agent
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
editing. When stdin is a pipe, the piped content is used as the description body — and if
`[TITLE]` is also omitted, the title is auto-extracted from the first `# heading` of the
piped content.

```text
gest task create [OPTIONS] [TITLE]
```

### Arguments

| Argument  | Description                                                                   |
|-----------|-------------------------------------------------------------------------------|
| `[TITLE]` | Task title (auto-extracted from the first `# heading` when piping stdin)      |

### Options

| Flag                              | Description                                                                         |
|-----------------------------------|-------------------------------------------------------------------------------------|
| `--assign <ASSIGN>`               | Assign the task to an author by name                                                |
| `--batch`                         | Read NDJSON from stdin (one task per line)                                          |
| `-d, --description <DESCRIPTION>` | Description text (opens `$EDITOR` if omitted and stdin is a terminal)               |
| `-i, --iteration <ITERATION>`     | Add the task to an iteration (ID or prefix)                                         |
| `-j, --json`                      | Output the created task as JSON                                                     |
| `-l, --link <LINK>`               | Create a link on the new task (repeatable, format: `<rel>:<target_id>`)             |
| `-m, --metadata <KEY=VALUE>`      | Set a metadata key=value pair (repeatable; supports dot-paths and scalar inference) |
| `--metadata-json <JSON>`          | Merge a JSON object into metadata (repeatable; applied after `--metadata` pairs)    |
| `--phase <PHASE>`                 | Phase within `--iteration` (requires `--iteration`; defaults to max + 1)            |
| `-p, --priority <PRIORITY>`       | Priority level (0-4, where 0 is highest)                                            |
| `-q, --quiet`                     | Print only the task ID                                                              |
| `-s, --status <STATUS>`           | Initial status: `open`, `in-progress`, `done`, or `cancelled` (default: `open`)     |
| `--tag <TAG>`                     | Tag (repeatable)                                                                    |

### Examples

```sh
# Create a simple task
gest task create "Implement login page"

# Create a task with description and tags
gest task create "Fix memory leak" -d "OOM after 24h uptime" --tag "bug,critical"

# Create a high-priority task assigned to an agent
gest task create "Write migration" -p 0 --assign agent

# Create a task and add it to an iteration with a link
gest task create "Add auth" -i iter123 -l child-of:spec456

# Create a task pinned to a specific phase of an iteration
gest task create "Add auth" -i iter123 --phase 2

# Pipe description from stdin
echo "Detailed description here" | gest task create "My task"

# Batch-create tasks from NDJSON
cat tasks.ndjson | gest task create --batch

# Machine-readable output
gest task create "Quick task" --json
gest task create "Quick task" -q
```

---

## task delete

Permanently delete a task and its dependent rows (notes, metadata, tags, links). By
default, a task that still belongs to one or more iterations will refuse to delete —
pass `--force` to drop its iteration memberships first.

```text
gest task delete [OPTIONS] <ID>
```

### Arguments

| Argument | Description              |
|----------|--------------------------|
| `<ID>`   | Task ID or unique prefix |

### Options

| Flag          | Description                                                                 |
|---------------|-----------------------------------------------------------------------------|
| `--yes`       | Skip the interactive confirmation prompt                                    |
| `--force`     | Remove the task from every iteration it belongs to before deleting          |
| `-j, --json`  | Output as JSON                                                              |
| `-q, --quiet` | Suppress normal output                                                      |

### Examples

```sh
# Interactive delete
gest task delete abc123

# Non-interactive
gest task delete abc123 --yes

# Delete even if task is still in iterations
gest task delete abc123 --yes --force
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
| `--limit <N>`                    | Cap the number of items returned                                |
| `-s, --status <STATUS>`          | Filter by status: `open`, `in-progress`, `done`, or `cancelled` |
| `-t, --tag <TAG>`                | Filter by tag                                                   |
| `-j, --json`                     | Output task list as JSON                                        |

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

| Flag                              | Description                                                                         |
|-----------------------------------|-------------------------------------------------------------------------------------|
| `--assigned-to <ASSIGNED_TO>`     | Set the assigned author by name                                                     |
| `-d, --description <DESCRIPTION>` | New description text                                                                |
| `-j, --json`                      | Output as JSON                                                                      |
| `-m, --metadata <KEY=VALUE>`      | Set a metadata key=value pair (repeatable; supports dot-paths and scalar inference) |
| `--metadata-json <JSON>`          | Merge a JSON object into metadata (repeatable; applied after `--metadata` pairs)    |
| `--phase <PHASE>`                 | Execution phase for parallel grouping                                               |
| `-p, --priority <PRIORITY>`       | Priority level (0-4, where 0 is highest)                                            |
| `-q, --quiet`                     | Print only the task ID                                                              |
| `-s, --status <STATUS>`           | New status (done/cancelled auto-resolves; open/in-progress un-resolves)             |
| `--tag <TAG>`                     | Replace all tags (repeatable)                                                       |
| `-t, --title <TITLE>`             | New title                                                                           |

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

## task unlink

Remove a relationship between a task and another task or artifact. Deletion is recorded
in the transaction log, so `gest undo` restores the edge.

```text
gest task unlink [OPTIONS] <ID> <TARGET>
```

### Arguments

| Argument   | Description                                  |
|------------|----------------------------------------------|
| `<ID>`     | Source task ID or unique prefix              |
| `<TARGET>` | Target task or artifact ID or unique prefix  |

### Options

| Flag                          | Description                                                                        |
|-------------------------------|------------------------------------------------------------------------------------|
| `--artifact`                  | Target is an artifact instead of a task                                            |
| `--rel <REL>`                 | Filter to relationships of this type. Required when multiple edges exist           |
| `--target-type <TARGET_TYPE>` | Target entity type: `task`, `artifact`, or `iteration` (default: `task`)           |
| `-j, --json`                  | Output the task as JSON after unlinking                                            |
| `-q, --quiet`                 | Output only the task ID                                                            |

If exactly one relationship exists between the source and target, `--rel` is optional.
If multiple relationships exist and `--rel` is omitted, the command exits with an error
listing the candidate rel-types. If no matching relationship exists, the command exits
with an error.

### Examples

```sh
# Remove a single relates-to edge (no --rel needed when unambiguous)
gest task unlink abc123 def456

# Remove the `blocks` edge when multiple edges exist between the pair
gest task unlink abc123 def456 --rel blocks

# Remove a link to an artifact
gest task unlink abc123 art789 --artifact
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

Add a note to a task. The `--body` flag is required; pass `-` to open `$EDITOR` for
interactive entry. Author defaults to `git config user.name` / `user.email`.

```text
gest task note add [OPTIONS] --body <BODY> <ID>
```

| Argument | Description              |
|----------|--------------------------|
| `<ID>`   | Task ID or unique prefix |

| Flag                | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| `--agent <AGENT>`   | Set the author (agent) identifier for this note                             |
| `-b, --body <BODY>` | Note body (required; use `-` to open `$EDITOR`)                             |
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

| Flag          | Description                      |
|---------------|----------------------------------|
| `--limit <N>` | Cap the number of items returned |
| `-j, --json`  | Output as JSON                   |
| `-q, --quiet` | Print only note IDs              |

### note show

Show a single note with full attribution and rendered markdown body.

```text
gest task note show [OPTIONS] <ID>
```

| Argument | Description              |
|----------|--------------------------|
| `<ID>`   | Note ID or unique prefix |

| Flag         | Description    |
|--------------|----------------|
| `-j, --json` | Output as JSON |

### note update

Update a note's body.

```text
gest task note update [OPTIONS] <ID>
```

| Argument | Description              |
|----------|--------------------------|
| `<ID>`   | Note ID or unique prefix |

| Flag                | Description                                 |
|---------------------|---------------------------------------------|
| `-b, --body <BODY>` | New body text (use `-` to open `$EDITOR`)   |
| `-j, --json`        | Output as JSON                              |
| `-q, --quiet`       | Print only the note ID                      |

### note delete

Delete a note from a task.

```text
gest task note delete [OPTIONS] <ID>
```

| Argument | Description              |
|----------|--------------------------|
| `<ID>`   | Note ID or unique prefix |

| Flag          | Description                              |
|---------------|------------------------------------------|
| `--yes`       | Skip the interactive confirmation prompt |
| `-j, --json`  | Output as JSON                           |
| `-q, --quiet` | Suppress normal output                   |

### Examples

```sh
# Add a human note (author from git config)
gest task note add abc123 --body "Found the root cause in the parser"

# Add an agent note
gest task note add abc123 --agent claude --body "Completed code review, no issues found"

# Open $EDITOR for the note body
gest task note add abc123 --body -

# List notes
gest task note list abc123

# Show a specific note (by note ID)
gest task note show nfkbqmrx

# Update a note
gest task note update nfkbqmrx --body "Updated analysis"

# Delete a note (non-interactive)
gest task note delete nfkbqmrx --yes
```
