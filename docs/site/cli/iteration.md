# gest iteration

Manage iterations -- execution plans that group tasks into phases. Iterations let you organize work into ordered phases for parallel or sequential execution.

## Usage

```
gest iteration <COMMAND> [OPTIONS]
```

## Subcommands

| Command | Description |
|---------|-------------|
| [`create`](#iteration-create) | Create a new iteration |
| [`list`](#iteration-list) | List iterations with optional filters |
| [`show`](#iteration-show) | Display an iteration's details |
| [`update`](#iteration-update) | Update an iteration's fields |
| [`add`](#iteration-add) | Add a task to an iteration |
| [`remove`](#iteration-remove) | Remove a task from an iteration |
| [`graph`](#iteration-graph) | Display the phased execution graph |
| [`tag`](#iteration-tag) | Add tags to an iteration |
| [`untag`](#iteration-untag) | Remove tags from an iteration |
| [`link`](#iteration-link) | Create a relationship between entities |
| [`meta`](#iteration-meta) | Read or write metadata fields |

---

## iteration create

Create a new iteration.

```
gest iteration create [OPTIONS] <TITLE>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<TITLE>` | Iteration title |

### Options

| Flag | Description |
|------|-------------|
| `-d, --description <DESCRIPTION>` | Description text |
| `-m, --metadata <METADATA>` | Key=value metadata pair (repeatable, e.g. `-m key=value`) |
| `-s, --status <STATUS>` | Initial status: `active`, `completed`, or `failed` (default: `active`) |
| `--tags <TAGS>` | Comma-separated list of tags |

### Examples

```sh
# Create a simple iteration
gest iteration create "Sprint 1"

# Create with description and tags
gest iteration create "Auth Refactor" -d "Rewrite authentication layer" --tags "backend,q2"
```

---

## iteration list

List iterations, optionally filtered by status or tag.

```
gest iteration list [OPTIONS]
```

### Options

| Flag | Description |
|------|-------------|
| `-a, --all` | Include resolved (completed/failed) iterations |
| `-j, --json` | Output iteration list as JSON |
| `-s, --status <STATUS>` | Filter by status: `active`, `completed`, or `failed` |
| `--tag <TAG>` | Filter by tag |

### Examples

```sh
gest iteration list
gest iteration list --all
gest iteration list -s active
```

---

## iteration show

Display an iteration's details, task counts, and phase summary.

```
gest iteration show [OPTIONS] <ID>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Iteration ID or unique prefix |

### Options

| Flag | Description |
|------|-------------|
| `-j, --json` | Output iteration details as JSON |

### Examples

```sh
gest iteration show abc123
gest iteration show abc123 --json
```

---

## iteration update

Update an iteration's title, description, status, tags, or metadata.

```
gest iteration update [OPTIONS] <ID>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Iteration ID or unique prefix |

### Options

| Flag | Description |
|------|-------------|
| `-d, --description <DESCRIPTION>` | New description |
| `-m, --metadata <METADATA>` | Key=value metadata pair, merged with existing (repeatable) |
| `-s, --status <STATUS>` | New status: `active`, `completed`, or `failed` |
| `--tags <TAGS>` | Replace all tags with this comma-separated list |
| `-t, --title <TITLE>` | New title |

### Examples

```sh
gest iteration update abc123 -s completed
gest iteration update abc123 -t "Sprint 1 - Revised" -m goal="deliver auth"
```

---

## iteration add

Add an existing task to an iteration.

```
gest iteration add <ID> <TASK_ID>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Iteration ID or unique prefix |
| `<TASK_ID>` | Task ID or unique prefix to add |

### Examples

```sh
gest iteration add iter123 task456
```

---

## iteration remove

Remove a task from an iteration.

```
gest iteration remove <ID> <TASK_ID>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Iteration ID or unique prefix |
| `<TASK_ID>` | Task ID or unique prefix to remove |

### Examples

```sh
gest iteration remove iter123 task456
```

---

## iteration graph

Display the phased execution graph for an iteration. This shows tasks grouped by phase with their statuses and dependencies.

```
gest iteration graph <ID>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Iteration ID or unique prefix |

### Examples

```sh
gest iteration graph abc123
```

---

## iteration tag

Add tags to an iteration.

```
gest iteration tag <ID> [TAGS]...
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Iteration ID or unique prefix |
| `[TAGS]...` | Tags to add (space-separated) |

### Examples

```sh
gest iteration tag abc123 sprint-1 backend
```

---

## iteration untag

Remove tags from an iteration.

```
gest iteration untag <ID> [TAGS]...
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Iteration ID or unique prefix |
| `[TAGS]...` | Tags to remove (space-separated) |

### Examples

```sh
gest iteration untag abc123 draft
```

---

## iteration link

Create a relationship between an iteration and another entity.

```
gest iteration link [OPTIONS] <ID> <REL> <TARGET_ID>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Iteration ID or unique prefix |
| `<REL>` | Relationship type: `blocked-by`, `blocks`, `child-of`, `parent-of`, `relates-to` |
| `<TARGET_ID>` | Target iteration or artifact ID or unique prefix |

### Options

| Flag | Description |
|------|-------------|
| `--artifact` | Target is an artifact instead of an iteration |

### Examples

```sh
gest iteration link abc123 blocks def456
gest iteration link abc123 relates-to art789 --artifact
```

---

## iteration meta

Read or write iteration metadata fields. Metadata uses dot-delimited key paths for nested values.

```
gest iteration meta <COMMAND>
```

### meta get

Retrieve a single metadata value.

```
gest iteration meta get <ID> <PATH>
```

| Argument | Description |
|----------|-------------|
| `<ID>` | Iteration ID or unique prefix |
| `<PATH>` | Dot-delimited key path (e.g. `outer.inner`) |

### meta set

Set a metadata value. Strings, numbers, and booleans are auto-detected.

```
gest iteration meta set <ID> <PATH> <VALUE>
```

| Argument | Description |
|----------|-------------|
| `<ID>` | Iteration ID or unique prefix |
| `<PATH>` | Dot-delimited key path (e.g. `outer.inner`) |
| `<VALUE>` | Value to set |

### Examples

```sh
gest iteration meta set abc123 goal "Ship auth module"
gest iteration meta get abc123 goal
```
