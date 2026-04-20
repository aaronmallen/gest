# gest project

Show or manage the project registration for the current working directory.

In gest v0.5.0, every tracked project is a row in the `projects` table, keyed on
its root path. `gest init` creates the row; `gest project` commands inspect and
modify it.

## Usage

```text
gest project [OPTIONS] [COMMAND]
```

### Options (bare `gest project`)

When run without a subcommand, `gest project` shows details for the current directory's
project.

| Flag     | Description                                                 |
|----------|-------------------------------------------------------------|
| `--json` | Emit output as JSON (only applies to the default show view) |

## Subcommands

| Command                            | Aliases | Description                                                        |
|------------------------------------|---------|--------------------------------------------------------------------|
| [`archive`](#project-archive)      |         | Soft-archive a project, hiding it from default list views          |
| [`attach`](#project-attach)        |         | Attach the current directory to an existing project as a workspace |
| [`delete`](#project-delete)        | `rm`    | Delete a project and all of its owned entities                     |
| [`detach`](#project-detach)        |         | Detach the current directory from its project                      |
| [`list`](#project-list)            | `ls`    | List all known projects                                            |
| [`unarchive`](#project-unarchive)  |         | Restore an archived project to active status                       |

Running `gest project` without a subcommand shows the current project.

---

## project archive

Soft-archive a project, detaching all workspaces and hiding owned entities
from default list views. The project and its data remain in the database and
can be restored with [`unarchive`](#project-unarchive).

A confirmation prompt shows the number of workspaces, tasks, iterations, and
artifacts that will be affected.

```text
gest project archive [OPTIONS] <ID>
```

### Arguments

| Argument | Description                 |
|----------|-----------------------------|
| `<ID>`   | Project ID or unique prefix |

### Options

| Flag    | Description                              |
|---------|------------------------------------------|
| `--yes` | Skip the interactive confirmation prompt |

### Examples

```sh
gest project archive abc123
gest project archive --yes abc123
```

---

## project attach

Attach the current directory to an existing project as a secondary workspace.
Useful when you have multiple checkouts of the same repository (for example, jj
workspaces or git worktrees) and want them all to share the same entity data.

```text
gest project attach <PROJECT-ID>
```

---

## project delete

Delete a project and all of its owned entities. This is a hard delete that
cascades through every task, iteration, and artifact owned by the project,
along with all their notes, tags, and relationships. Tombstone files are
written so downstream sync imports do not resurrect deleted data.

This operation is **not** reversible via `gest undo`.

```text
gest project delete [OPTIONS] <ID>
```

### Arguments

| Argument | Description                 |
|----------|-----------------------------|
| `<ID>`   | Project ID or unique prefix |

### Options

| Flag    | Description                              |
|---------|------------------------------------------|
| `--yes` | Skip the interactive confirmation prompt |

### Examples

```sh
gest project delete abc123
gest project rm --yes abc123
```

---

## project detach

Remove the current directory from its project's workspace list. The project
itself is not deleted — only the workspace association.

```text
gest project detach
```

---

## project list

List every project recorded in the database. Archived projects are hidden by
default.

```text
gest project list [OPTIONS]
```

### Options

| Flag            | Description                              |
|-----------------|------------------------------------------|
| `-a, --all`     | Include archived projects in the listing |
| `--limit <N>`   | Cap the number of items returned         |
| `-j, --json`    | Emit output as JSON                      |
| `-q, --quiet`   | Suppress normal output                   |
| `-r, --raw`     | Emit script-friendly plain output        |

### Examples

```sh
gest project list
gest project list --all
gest project list --json
gest project list --limit 5
```

---

## project unarchive

Restore an archived project to active status by clearing its `archived_at`
timestamp. Workspace paths are not automatically restored — you will need to
re-attach them with [`gest project attach`](#project-attach).

```text
gest project unarchive <ID>
```

### Arguments

| Argument | Description                 |
|----------|-----------------------------|
| `<ID>`   | Project ID or unique prefix |

### Examples

```sh
gest project unarchive abc123
```

---

## Examples

Show the project for the current directory:

```sh
gest project
```

List every project:

```sh
gest project list
```

Attach a secondary checkout:

```sh
cd ../myapp-workspace-2
gest project attach <project-id>
```
