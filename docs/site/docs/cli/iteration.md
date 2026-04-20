---
title: gest iteration
description: gest iteration — group tasks into phased execution plans for parallel or sequential runs across multiple AI agent workspaces.
keywords: [gest iteration CLI, phased execution plan, parallel AI agents, task phases, agentic workflow CLI]
---

# gest iteration

Manage iterations -- execution plans that group tasks into phases. Iterations let you
organize work into ordered phases for parallel or sequential execution.

## Usage

```text
gest iteration <COMMAND> [OPTIONS]
```

## Subcommands

| Command                           | Aliases | Description                                          |
|-----------------------------------|---------|------------------------------------------------------|
| [`add`](#iteration-add)           |         | Add a task to an iteration                           |
| [`advance`](#iteration-advance)   |         | Advance to the next phase                            |
| [`cancel`](#iteration-cancel)     |         | Cancel an iteration and all its non-terminal tasks   |
| [`complete`](#iteration-complete) |         | Mark an iteration as completed                       |
| [`create`](#iteration-create)     | `new`   | Create a new iteration                               |
| [`delete`](#iteration-delete)     |         | Delete an iteration and drop its task memberships    |
| [`graph`](#iteration-graph)       |         | Display the phased execution graph                   |
| [`link`](#iteration-link)         |         | Create a relationship between entities               |
| [`unlink`](#iteration-unlink)     |         | Remove a relationship between entities               |
| [`list`](#iteration-list)         | `ls`    | List iterations with optional filters                |
| [`meta`](#iteration-meta)         |         | Read or write metadata fields                        |
| [`next`](#iteration-next)         |         | Find or claim the next available task                |
| [`remove`](#iteration-remove)     | `rm`    | Remove a task from an iteration                      |
| [`reopen`](#iteration-reopen)     |         | Reopen a completed or cancelled iteration            |
| [`show`](#iteration-show)         | `view`  | Display an iteration's details                       |
| [`status`](#iteration-status)     |         | Display aggregated iteration progress                |
| [`tag`](#iteration-tag)           |         | Add tags to an iteration                             |
| [`untag`](#iteration-untag)       |         | Remove tags from an iteration                        |
| [`update`](#iteration-update)     | `edit`  | Update an iteration's fields                         |

## Exit Codes

| Code | When                                                                         |
|------|------------------------------------------------------------------------------|
| 0    | Success                                                                      |
| 64   | Bad flags, malformed NDJSON batch input, or bad relationship spec            |
| 65   | Could not serialize to or deserialize from JSON/TOML                         |
| 66   | Iteration, task, or metadata key did not resolve                             |
| 69   | State conflict (e.g. `advance` on a terminal iteration, `next` on inactive)  |
| 70   | Editor launch, non-zero editor exit, or empty required body                  |
| 74   | Store I/O error                                                              |
| 75   | `iteration next`: no unblocked tasks available in the active phase           |
| 78   | Not a gest project (run `gest init`)                                         |

See [Exit Codes](./exit-codes.md) for the full contract. `iteration next` has
a narrower subcommand-specific table in its section below.

---

## iteration add

Add an existing task to an iteration. In single mode, one task is added per
invocation. In batch mode, NDJSON records are read from stdin.

```text
gest iteration add [OPTIONS] <ID> [TASK_ID]
```

### Arguments

| Argument    | Description                                                |
|-------------|------------------------------------------------------------|
| `<ID>`      | Iteration ID or unique prefix                              |
| `[TASK_ID]` | Task ID or unique prefix to add (conflicts with `--batch`) |

### Options

| Flag                  | Description                                                                          |
|-----------------------|--------------------------------------------------------------------------------------|
| `--batch`             | Read NDJSON task records from stdin (conflicts with `TASK_ID` and `--phase`)         |
| `-j, --json`          | Output as JSON                                                                       |
| `-p, --phase <PHASE>` | Phase to add the task to (defaults to next phase, max + 1; conflicts with `--batch`) |
| `-q, --quiet`         | Output only the task or iteration ID                                                 |

### Batch NDJSON schema

Each line must be a JSON object with these fields:

| Field   | Type   | Required | Description                                                 |
|---------|--------|----------|-------------------------------------------------------------|
| `task`  | string | yes      | Task ID or unique prefix                                    |
| `phase` | number | no       | Phase to assign (auto-increments from max + 1 when omitted) |

### Examples

```sh
# Append to the next phase (max + 1)
gest iteration add iter123 task456

# Pin to an explicit phase
gest iteration add iter123 task456 --phase 2

# Batch-add from NDJSON stdin
cat <<'EOF' | gest iteration add iter123 --batch
{"task":"task456","phase":1}
{"task":"task789","phase":2}
{"task":"taskABC"}
EOF
```

---

## iteration advance

Validate that the active phase is complete and advance to the next phase. All tasks in the
current phase must be in a terminal state (done or cancelled) unless `--force` is used.

```text
gest iteration advance [OPTIONS] <ID>
```

### Arguments

| Argument | Description                   |
|----------|-------------------------------|
| `<ID>`   | Iteration ID or unique prefix |

### Options

| Flag      | Description                                              |
|-----------|----------------------------------------------------------|
| `--force` | Advance even if the current phase has non-terminal tasks |

### Examples

```sh
# Advance after all phase tasks are done
gest iteration advance abc123

# Force-advance past incomplete tasks
gest iteration advance abc123 --force
```

---

## iteration cancel

Cancel an iteration and automatically cancel all its non-terminal tasks (`open` and
`in-progress`). Tasks already `done` or `cancelled` are not affected. This is a shortcut
for `iteration update <ID> --status cancelled`.

```text
gest iteration cancel [OPTIONS] <ID>
```

### Arguments

| Argument | Description                   |
|----------|-------------------------------|
| `<ID>`   | Iteration ID or unique prefix |

### Options

| Flag          | Description       |
|---------------|-------------------|
| `-j, --json`  | Output as JSON    |
| `-q, --quiet` | Print only the ID |

### Examples

```sh
# Cancel an iteration and all its open tasks
gest iteration cancel abc123

# Cancel with JSON output
gest iteration cancel abc123 --json
```

---

## iteration complete

Mark an iteration as completed. This is a shortcut for `iteration update <ID> --status completed`
(though `update` no longer accepts `--status` directly — use `complete` or `cancel` instead).

```text
gest iteration complete [OPTIONS] <ID>
```

### Arguments

| Argument | Description                   |
|----------|-------------------------------|
| `<ID>`   | Iteration ID or unique prefix |

### Options

| Flag          | Description       |
|---------------|-------------------|
| `-j, --json`  | Output as JSON    |
| `-q, --quiet` | Print only the ID |

### Examples

```sh
gest iteration complete abc123
gest iteration complete abc123 --json
```

---

## iteration create

Create a new iteration.

```text
gest iteration create [OPTIONS] <TITLE>
```

### Arguments

| Argument  | Description     |
|-----------|-----------------|
| `<TITLE>` | Iteration title |

### Options

| Flag                              | Description                                                                         |
|-----------------------------------|-------------------------------------------------------------------------------------|
| `-d, --description <DESCRIPTION>` | Description text                                                                    |
| `-j, --json`                      | Output the created iteration as JSON                                                |
| `-m, --metadata <KEY=VALUE>`      | Set a metadata key=value pair (repeatable; supports dot-paths and scalar inference) |
| `--metadata-json <JSON>`          | Merge a JSON object into metadata (repeatable; applied after `--metadata` pairs)    |
| `-q, --quiet`                     | Print only the iteration ID                                                         |
| `-s, --status <STATUS>`           | Initial status: `active`, `cancelled`, or `completed` (default: `active`)           |
| `-t, --tag <TAG>`                 | Tag (repeatable)                                                                    |

### Examples

```sh
# Create a simple iteration
gest iteration create "Sprint 1"

# Create with description and tags
gest iteration create "Auth Refactor" -d "Rewrite authentication layer" --tag "backend,q2"

# Machine-readable output
gest iteration create "Sprint 2" --json
gest iteration create "Sprint 2" -q
```

---

## iteration delete

Permanently delete an iteration and drop its task memberships. Tasks themselves are not
deleted; they just lose their iteration association. This is irreversible.

```text
gest iteration delete [OPTIONS] <ID>
```

### Arguments

| Argument | Description                   |
|----------|-------------------------------|
| `<ID>`   | Iteration ID or unique prefix |

### Options

| Flag          | Description                                                                     |
|---------------|---------------------------------------------------------------------------------|
| `--yes`       | Skip the interactive confirmation prompt                                        |
| `--force`     | Reserved for future guards; currently a no-op (iterations have no guards today) |
| `-j, --json`  | Output as JSON                                                                  |
| `-q, --quiet` | Suppress normal output                                                          |

### Examples

```sh
# Interactive (prompts for confirmation)
gest iteration delete abc123

# Non-interactive
gest iteration delete abc123 --yes
```

---

## iteration graph

Display the phased execution graph for an iteration. This shows tasks grouped by phase
with their statuses and dependencies.

```text
gest iteration graph <ID>
```

### Arguments

| Argument | Description                   |
|----------|-------------------------------|
| `<ID>`   | Iteration ID or unique prefix |

### Examples

```sh
gest iteration graph abc123
```

---

## iteration link

Create a relationship between an iteration and another entity. For iteration-to-iteration
links a reciprocal row is written automatically, so linking `A blocks B` also records
`B blocked-by A`.

```text
gest iteration link [OPTIONS] <ID> <TARGET> [--rel <REL>]
```

### Arguments

| Argument   | Description                                      |
|------------|--------------------------------------------------|
| `<ID>`     | Iteration ID or unique prefix                    |
| `<TARGET>` | Target iteration or artifact ID or unique prefix |

### Options

| Flag          | Description                                                                                              |
|---------------|----------------------------------------------------------------------------------------------------------|
| `--artifact`  | Target is an artifact instead of an iteration                                                            |
| `--rel <REL>` | Relationship type: `blocked-by`, `blocks`, `child-of`, `parent-of`, `relates-to` (default: `relates-to`) |
| `-j, --json`  | Output the iteration as JSON after linking                                                               |
| `-q, --quiet` | Output only the iteration ID                                                                             |

### Examples

```sh
gest iteration link abc123 def456 --rel blocks
gest iteration link abc123 art789 --artifact --rel relates-to
```

:::caution Deprecated: positional `<REL>`

Earlier releases accepted a positional `<REL>` argument between `<ID>` and `<TARGET>`:

```sh
# Deprecated; still works but emits a warning
gest iteration link abc123 blocks def456
```

This form is still accepted for backward compatibility but prints a deprecation warning
to stderr and will be removed in a future major version. Prefer `--rel <type>`.
:::

---

## iteration unlink

Remove a relationship between an iteration and another entity. For iteration-to-iteration
edges, both the named row and its reciprocal are deleted atomically in a single
transaction, mirroring how `iteration link` creates both halves. `gest undo` restores
the deleted edges.

```text
gest iteration unlink [OPTIONS] <ID> <TARGET>
```

### Arguments

| Argument   | Description                                      |
|------------|--------------------------------------------------|
| `<ID>`     | Iteration ID or unique prefix                    |
| `<TARGET>` | Target iteration or artifact ID or unique prefix |

### Options

| Flag          | Description                                                                    |
|---------------|--------------------------------------------------------------------------------|
| `--artifact`  | Target is an artifact instead of an iteration                                  |
| `--rel <REL>` | Filter to relationships of this type. Required when multiple edges exist       |
| `-j, --json`  | Output the iteration as JSON after unlinking                                   |
| `-q, --quiet` | Output only the iteration ID                                                   |

If exactly one relationship exists between the source and target, `--rel` is optional.
If multiple exist and `--rel` is omitted, the command exits with an error listing the
candidate rel-types. If no matching relationship exists, the command exits with an error.

### Examples

```sh
# Remove an iteration-to-iteration blocks edge (also removes the reciprocal blocked-by row)
gest iteration unlink abc123 def456 --rel blocks

# Remove a link to an artifact
gest iteration unlink abc123 art789 --artifact
```

---

## iteration list

List iterations, optionally filtered by status or tag.

```text
gest iteration list [OPTIONS]
```

### Options

| Flag                    | Description                                                          |
|-------------------------|----------------------------------------------------------------------|
| `-a, --all`             | Include resolved (completed/cancelled) iterations                    |
| `--has-available`       | Only show iterations with at least one claimable task                |
| `--limit <N>`           | Cap the number of items returned                                     |
| `-s, --status <STATUS>` | Filter by status: `active`, `cancelled`, or `completed`              |
| `-t, --tag <TAG>`       | Filter by tag                                                        |
| `-j, --json`            | Output iteration list as JSON                                        |

### Examples

```sh
gest iteration list
gest iteration list --all
gest iteration list -s active
```

---

## iteration meta

Read or write iteration metadata fields. Metadata uses dot-delimited key paths for nested values.

```text
gest iteration meta <COMMAND>
```

### meta get

Retrieve a single metadata value.

```text
gest iteration meta get [OPTIONS] <ID> <PATH>
```

| Argument | Description                                 |
|----------|---------------------------------------------|
| `<ID>`   | Iteration ID or unique prefix               |
| `<PATH>` | Dot-delimited key path (e.g. `outer.inner`) |

| Flag     | Description                           |
|----------|---------------------------------------|
| `--json` | Output as a JSON object               |
| `--raw`  | Output the bare value with no styling |

### meta set

Set a metadata value. Strings, numbers, and booleans are auto-detected.

```text
gest iteration meta set [OPTIONS] <ID> <PATH> <VALUE>
```

| Argument  | Description                                 |
|-----------|---------------------------------------------|
| `<ID>`    | Iteration ID or unique prefix               |
| `<PATH>`  | Dot-delimited key path (e.g. `outer.inner`) |
| `<VALUE>` | Value to set                                |

| Flag          | Description              |
|---------------|--------------------------|
| `-j, --json`  | Output as JSON           |
| `-q, --quiet` | Print only the entity ID |

### Examples

```sh
# Set a metadata field
gest iteration meta set abc123 goal "Ship auth module"

# Read it back
gest iteration meta get abc123 goal

# JSON output
gest iteration meta get abc123 goal --json

# Raw value (no styling)
gest iteration meta get abc123 goal --raw
```

---

## iteration next

Find (or claim) the next available task in an iteration. Candidates are drawn from the
active phase (the lowest phase with incomplete tasks) and sorted by phase ascending, then
by priority ascending (lower number = higher priority). No further tie-break is applied.

```text
gest iteration next [OPTIONS] <ID>
```

### Arguments

| Argument | Description                   |
|----------|-------------------------------|
| `<ID>`   | Iteration ID or unique prefix |

### Options

| Flag              | Description                                                   |
|-------------------|---------------------------------------------------------------|
| `--claim`         | Set the task to in-progress and assign it (may be used alone) |
| `--agent <AGENT>` | Agent name for assignment (requires `--claim`)                |
| `-j, --json`      | Output as JSON                                                |
| `-q, --quiet`     | Print only the task ID                                        |

### Exit Codes

Gest follows the BSD [`sysexits.h`](https://man.openbsd.org/sysexits.3)
convention. `iteration next` returns one of:

| Code | Name             | Meaning                                            |
|------|------------------|----------------------------------------------------|
| 0    | —                | Task found (and claimed if `--claim` was used)     |
| 64   | `EX_USAGE`       | Bad flags (e.g. `--agent` without `--claim`)       |
| 66   | `EX_NOINPUT`     | Iteration ID did not resolve                       |
| 69   | `EX_UNAVAILABLE` | Iteration is not active                            |
| 75   | `EX_TEMPFAIL`    | No unblocked tasks are available in active phase   |

### Examples

```sh
# Peek at the next task without claiming
gest iteration next abc123

# Claim the next task for an agent
gest iteration next abc123 --claim --agent worker-1

# Machine-readable output
gest iteration next abc123 --claim --agent worker-1 --json
```

---

## iteration remove

Remove a task from an iteration.

```text
gest iteration remove [OPTIONS] <ID> <TASK_ID>
```

### Arguments

| Argument    | Description                        |
|-------------|------------------------------------|
| `<ID>`      | Iteration ID or unique prefix      |
| `<TASK_ID>` | Task ID or unique prefix to remove |

### Options

| Flag          | Description                                           |
|---------------|-------------------------------------------------------|
| `-j, --json`  | Output the iteration as JSON after removing the task  |
| `-q, --quiet` | Output only the iteration ID                          |

### Examples

```sh
gest iteration remove iter123 task456
```

---

## iteration reopen

Reopen a completed or cancelled iteration and restore all its cancelled tasks to `open`.
Tasks with `done` status are left unchanged. This reverses the effect of
`iteration cancel` or `iteration complete`.

```text
gest iteration reopen [OPTIONS] <ID>
```

### Arguments

| Argument | Description                   |
|----------|-------------------------------|
| `<ID>`   | Iteration ID or unique prefix |

### Options

| Flag          | Description       |
|---------------|-------------------|
| `-j, --json`  | Output as JSON    |
| `-q, --quiet` | Print only the ID |

### Examples

```sh
# Reopen a cancelled iteration
gest iteration reopen abc123

# Reopen with JSON output
gest iteration reopen abc123 --json
```

---

## iteration show

Display an iteration's details, task counts, and phase summary.

```text
gest iteration show [OPTIONS] <ID>
```

### Arguments

| Argument | Description                   |
|----------|-------------------------------|
| `<ID>`   | Iteration ID or unique prefix |

### Options

| Flag         | Description                      |
|--------------|----------------------------------|
| `-j, --json` | Output iteration details as JSON |

### Examples

```sh
gest iteration show abc123
gest iteration show abc123 --json
```

---

## iteration status

Display aggregated progress for an iteration, including active phase, task counts,
blockers, and assignees.

```text
gest iteration status [OPTIONS] <ID>
```

### Arguments

| Argument | Description                   |
|----------|-------------------------------|
| `<ID>`   | Iteration ID or unique prefix |

### Options

| Flag         | Description                     |
|--------------|---------------------------------|
| `-j, --json` | Output iteration status as JSON |

### Examples

```sh
gest iteration status abc123
gest iteration status abc123 --json
```

---

## iteration tag

Add tags to an iteration, deduplicating with any existing tags.

```text
gest iteration tag [OPTIONS] <ID> [TAGS]...
```

### Arguments

| Argument    | Description                            |
|-------------|----------------------------------------|
| `<ID>`      | Iteration ID or unique prefix          |
| `[TAGS]...` | Tags to add (space or comma-separated) |

### Options

| Flag          | Description                                |
|---------------|--------------------------------------------|
| `-j, --json`  | Output the iteration as JSON after tagging |
| `-q, --quiet` | Output only the iteration ID               |

### Examples

```sh
gest iteration tag abc123 sprint-1 backend
gest iteration tag abc123 sprint-1,backend
```

---

## iteration untag

Remove tags from an iteration.

```text
gest iteration untag [OPTIONS] <ID> [TAGS]...
```

### Arguments

| Argument    | Description                               |
|-------------|-------------------------------------------|
| `<ID>`      | Iteration ID or unique prefix             |
| `[TAGS]...` | Tags to remove (space or comma-separated) |

### Options

| Flag          | Description                                  |
|---------------|----------------------------------------------|
| `-j, --json`  | Output the iteration as JSON after untagging |
| `-q, --quiet` | Output only the iteration ID                 |

### Examples

```sh
gest iteration untag abc123 draft
```

---

## iteration update

Update an iteration's title, description, or metadata. For status changes, use the
[`complete`](#iteration-complete), [`cancel`](#iteration-cancel), or
[`reopen`](#iteration-reopen) shortcuts instead. Tag changes go through
[`tag`](#iteration-tag) / [`untag`](#iteration-untag).

```text
gest iteration update [OPTIONS] <ID>
```

### Arguments

| Argument | Description                   |
|----------|-------------------------------|
| `<ID>`   | Iteration ID or unique prefix |

### Options

| Flag                              | Description                                                                         |
|-----------------------------------|-------------------------------------------------------------------------------------|
| `-d, --description <DESCRIPTION>` | New description                                                                     |
| `-j, --json`                      | Output as JSON                                                                      |
| `-m, --metadata <KEY=VALUE>`      | Set a metadata key=value pair (repeatable; supports dot-paths and scalar inference) |
| `--metadata-json <JSON>`          | Merge a JSON object into metadata (repeatable; applied after `--metadata` pairs)    |
| `-q, --quiet`                     | Print only the iteration ID                                                         |
| `-t, --title <TITLE>`             | New title                                                                           |

### Examples

```sh
gest iteration update abc123 -t "Sprint 1 - Revised"
gest iteration update abc123 -m goal="deliver auth"

# Machine-readable output
gest iteration update abc123 -t "Sprint 1 - Revised" --json
```
