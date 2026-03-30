# gest task

Create, update, list, and manage tasks. Tasks represent units of work with a title, description, status, priority, tags, metadata, and relationship links.

## Usage

```
gest task <COMMAND> [OPTIONS]
```

## Subcommands

| Command | Description |
|---------|-------------|
| [`create`](#task-create) | Create a new task |
| [`list`](#task-list) | List tasks with optional filters |
| [`show`](#task-show) | Display a task's full details |
| [`update`](#task-update) | Update a task's fields |
| [`tag`](#task-tag) | Add tags to a task |
| [`untag`](#task-untag) | Remove tags from a task |
| [`link`](#task-link) | Create a relationship between entities |
| [`meta`](#task-meta) | Read or write metadata fields |

---

## task create

Create a new task with optional metadata, tags, and status.

```
gest task create [OPTIONS] <TITLE>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<TITLE>` | Task title |

### Options

| Flag | Description |
|------|-------------|
| `--assigned-to <ASSIGNED_TO>` | Actor assigned to this task |
| `-d, --description <DESCRIPTION>` | Description text (opens `$EDITOR` if omitted and stdin is a terminal) |
| `-m, --metadata <METADATA>` | Key=value metadata pair (repeatable, e.g. `-m key=value`) |
| `--phase <PHASE>` | Execution phase for parallel grouping |
| `-p, --priority <PRIORITY>` | Priority level (0-4, where 0 is highest) |
| `-s, --status <STATUS>` | Initial status: `open`, `in-progress`, `done`, or `cancelled` (default: `open`) |
| `--tags <TAGS>` | Comma-separated list of tags |

### Examples

```sh
# Create a simple task
gest task create "Implement login page"

# Create a task with description and tags
gest task create "Fix memory leak" -d "OOM after 24h uptime" --tags "bug,critical"

# Create a high-priority task assigned to an agent
gest task create "Write migration" -p 0 --assigned-to agent --phase 1
```

---

## task list

List tasks, optionally filtered by status or tag.

```
gest task list [OPTIONS]
```

### Options

| Flag | Description |
|------|-------------|
| `-a, --all` | Include resolved (done/cancelled) tasks |
| `-j, --json` | Output task list as JSON |
| `-s, --status <STATUS>` | Filter by status: `open`, `in-progress`, `done`, or `cancelled` |
| `--tag <TAG>` | Filter by tag |

### Examples

```sh
# List active tasks
gest task list

# List all tasks including resolved
gest task list --all

# Filter by status
gest task list -s in-progress

# JSON output for scripting
gest task list --json
```

---

## task show

Display a task's full details, description, and links.

```
gest task show [OPTIONS] <ID>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Task ID or unique prefix |

### Options

| Flag | Description |
|------|-------------|
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

```
gest task update [OPTIONS] <ID>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Task ID or unique prefix |

### Options

| Flag | Description |
|------|-------------|
| `--assigned-to <ASSIGNED_TO>` | Actor assigned to this task |
| `-d, --description <DESCRIPTION>` | New description text |
| `-m, --metadata <METADATA>` | Key=value metadata pair, merged with existing (repeatable) |
| `--phase <PHASE>` | Execution phase for parallel grouping |
| `-p, --priority <PRIORITY>` | Priority level (0-4, where 0 is highest) |
| `-s, --status <STATUS>` | New status (done/cancelled auto-resolves; open/in-progress un-resolves) |
| `--tags <TAGS>` | Replace all tags with this comma-separated list |
| `-t, --title <TITLE>` | New title |

### Examples

```sh
# Mark a task as done
gest task update abc123 -s done

# Update title and description
gest task update abc123 -t "New title" -d "Updated description"

# Add metadata
gest task update abc123 -m estimate=3h -m complexity=high
```

---

## task tag

Add tags to a task, deduplicating with any existing tags.

```
gest task tag <ID> [TAGS]...
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Task ID or unique prefix |
| `[TAGS]...` | Tags to add (space-separated) |

### Examples

```sh
gest task tag abc123 bug critical
```

---

## task untag

Remove tags from a task.

```
gest task untag <ID> [TAGS]...
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Task ID or unique prefix |
| `[TAGS]...` | Tags to remove (space-separated) |

### Examples

```sh
gest task untag abc123 critical
```

---

## task link

Create a relationship between a task and another task or artifact.

```
gest task link [OPTIONS] <ID> <REL> <TARGET_ID>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Source task ID or unique prefix |
| `<REL>` | Relationship type: `blocked-by`, `blocks`, `child-of`, `parent-of`, `relates-to` |
| `<TARGET_ID>` | Target task or artifact ID or unique prefix |

### Options

| Flag | Description |
|------|-------------|
| `--artifact` | Target is an artifact instead of a task (no reciprocal link is created) |

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

```
gest task meta <COMMAND>
```

### meta get

Retrieve a single metadata value.

```
gest task meta get <ID> <PATH>
```

| Argument | Description |
|----------|-------------|
| `<ID>` | Task ID or unique prefix |
| `<PATH>` | Dot-delimited key path (e.g. `outer.inner`) |

### meta set

Set a metadata value. Strings, numbers, and booleans are auto-detected.

```
gest task meta set <ID> <PATH> <VALUE>
```

| Argument | Description |
|----------|-------------|
| `<ID>` | Task ID or unique prefix |
| `<PATH>` | Dot-delimited key path (e.g. `outer.inner`) |
| `<VALUE>` | Value to set |

### Examples

```sh
# Set a metadata field
gest task meta set abc123 estimate "3 hours"

# Read it back
gest task meta get abc123 estimate
```
