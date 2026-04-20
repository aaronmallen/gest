# gest tag

Add, remove, and list tags across tasks, artifacts, and iterations without knowing
the entity type in advance. The entity is resolved automatically from the ID prefix.

> [!NOTE]
> This command replaces the previous `gest tags` command, which only supported listing.

## Usage

```text
gest tag [OPTIONS] [COMMAND]
```

Running `gest tag` without a subcommand is equivalent to `gest tag list`.

## Subcommands

| Command                 | Description                                              |
|-------------------------|----------------------------------------------------------|
| [`add`](#tag-add)       | Add tags to any entity by ID prefix                      |
| [`remove`](#tag-remove) | Remove tags from any entity by ID prefix                 |
| [`list`](#tag-list)     | List all unique tags, optionally filtered by entity type |

---

## tag add

Add one or more tags to an entity. The ID prefix is resolved across tasks, artifacts,
and iterations. If the prefix is ambiguous (matches multiple entity types), an error
is returned with disambiguation guidance.

```text
gest tag add <ID> [TAGS]...
```

### Arguments

| Argument    | Description                            |
|-------------|----------------------------------------|
| `<ID>`      | Entity ID or unique prefix             |
| `[TAGS]...` | Tags to add (space or comma-separated) |

### Examples

```sh
gest tag add zyxw rust cli    # tag a task (or artifact/iteration) with "rust" and "cli"
gest tag add zyxw rust,cli    # comma-separated form
```

---

## tag remove

Remove one or more tags from an entity. Uses the same cross-entity resolution as `tag add`.

```text
gest tag remove <ID> [TAGS]...
```

### Arguments

| Argument    | Description                               |
|-------------|-------------------------------------------|
| `<ID>`      | Entity ID or unique prefix                |
| `[TAGS]...` | Tags to remove (space or comma-separated) |

### Examples

```sh
gest tag remove zyxw cli      # remove the "cli" tag from the matched entity
```

---

## tag list

List all unique tags in the project. By default, tags are collected from all entity types.
Use the `--task`, `--artifact`, or `--iteration` flags to filter by entity type.

```text
gest tag list [OPTIONS]
```

### Options

| Option        | Description                              |
|---------------|------------------------------------------|
| `--task`      | Show only tags from tasks                |
| `--artifact`  | Show only tags from artifacts            |
| `--iteration` | Show only tags from iterations           |
| `--limit <N>` | Cap the number of items returned         |
| `-j, --json`  | Emit output as JSON                      |
| `-q, --quiet` | Suppress normal output                   |
| `-r, --raw`   | Emit script-friendly plain output        |

Entity-scope flags can be combined — `--task --artifact` shows tags from both tasks and artifacts.

### Examples

```sh
gest tag list                    # all tags across all entity types
gest tag list --task             # only tags used on tasks
gest tag list --task --artifact  # tags from tasks and artifacts
gest tag list --limit 20 --json  # machine-readable output, capped
```
