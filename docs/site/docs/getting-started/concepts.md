---
title: Core Concepts
description: Core concepts in gest — tasks, artifacts, iterations, links, and tags — and how they connect into phased execution plans for parallel AI agents.
keywords: [gest concepts, tasks, artifacts, iterations, phased execution plan, parallel AI agents]
---

# Core Concepts

Gest manages three primary entity types -- tasks, artifacts, and iterations -- and connects them
through links, tags, and metadata. Entity data lives in a local SQLite database (via libsql); for
local-mode projects a sync layer mirrors every change to a `.gest/` directory as YAML and Markdown
files so you can commit the data alongside your code.

## Tasks

A **task** represents a unit of work. Tasks are rows in the `tasks` table with the following
columns (and their YAML mirror in `.gest/task/<id>.yaml` when local sync is enabled):

| Field         | Description                                   |
|---------------|-----------------------------------------------|
| `title`       | Short summary of the work                     |
| `description` | Longer explanation (Markdown)                 |
| `status`      | Current state of the task                     |
| `priority`    | Urgency level, 0 (highest) through 4 (lowest) |
| `phase`       | Numeric execution phase for parallel grouping |
| `assigned_to` | Actor responsible for the task                |
| `tags`        | Freeform labels for filtering and grouping    |
| `metadata`    | Arbitrary key-value pairs (JSON object)       |
| `links`       | Relationships to other tasks or artifacts     |

### Task Statuses

| Status        | Meaning                       |
|---------------|-------------------------------|
| `open`        | Not yet started (default)     |
| `in-progress` | Actively being worked on      |
| `done`        | Completed successfully        |
| `cancelled`   | Abandoned or no longer needed |

`done` and `cancelled` are terminal statuses. Resolved tasks are excluded from listings and
searches by default; pass `--all` to include them.

### Priority

Priority is a number from `0` to `4`, where `0` is the most urgent. Priority is optional;
tasks without a priority are treated as unprioritized rather than defaulting to any level.

### Phase

Phase is a numeric label used to group tasks for parallel execution inside an iteration. Tasks
in the same phase have no ordering dependency on each other and can run concurrently. Lower
phase numbers execute first. Phases are the parallelism boundary: if one task blocks another,
they must live in different phases.

## Artifacts

An **artifact** is a document -- a spec, ADR, RFC, design note, or any other prose output
generated during development. Artifacts are rows in the `artifacts` table; when local sync is
enabled they are also mirrored to `.gest/artifact/<id>.md` as Markdown with YAML frontmatter.

| Field         | Description                                            |
|---------------|--------------------------------------------------------|
| `title`       | Document title (extracted from `# heading` if not set) |
| `body`        | Markdown content                                       |
| `tags`        | Freeform labels for filtering                          |
| `metadata`    | Arbitrary key-value pairs (JSON object)                |
| `archived_at` | Timestamp set when the artifact is archived            |

### Categorizing artifacts

Artifact categorization is tag-driven. Tag an artifact with `spec`, `adr`, `rfc`, `note`, or any
other label that fits your workflow, then filter with `--tag`:

```sh
gest artifact create "Auth spec" --tag spec --body "..."
gest artifact list --tag spec
```

Common conventions used in this project's own artifacts:

| Tag    | Description                      |
|--------|----------------------------------|
| `spec` | Product or feature specification |
| `adr`  | Architecture Decision Record     |
| `rfc`  | Request for Comments             |
| `note` | General-purpose document         |

### Archiving

Artifacts can be archived with `gest artifact archive <id>`. Archived artifacts are hidden from
listings and searches by default, but remain in the database. Use `--all` to include them in
queries.

## Iterations

An **iteration** groups related tasks into an execution plan. Iterations are rows in the
`iterations` table; the `iteration_tasks` join table records which tasks belong to which
iteration and at which phase.

| Field         | Description                                    |
|---------------|------------------------------------------------|
| `title`       | Name of the iteration                          |
| `description` | Goal or scope of the iteration (Markdown)      |
| `status`      | Current state of the iteration                 |
| `tasks`       | List of task IDs that belong to this iteration |
| `tags`        | Freeform labels                                |
| `metadata`    | Arbitrary key-value pairs (JSON object)        |
| `links`       | Relationships to other entities                |

### Iteration Statuses

| Status      | Meaning                              |
|-------------|--------------------------------------|
| `active`    | Currently in progress (default)      |
| `cancelled` | Iteration was deliberately abandoned |
| `completed` | All tasks finished successfully      |

### Dependency Graphs

Use `gest iteration graph <id>` to visualize the phased execution plan. The graph shows tasks
grouped by phase, with dependency edges derived from `blocked-by` / `blocks` links between
tasks. This makes it clear which tasks can run in parallel and which must wait.

### Managing Tasks in an Iteration

```sh
gest iteration add <iteration-id> <task-id>      # add a task
gest iteration remove <iteration-id> <task-id>   # remove a task
```

## Working across workspaces

Gest's zero-config discovery resolves each checkout to its own project row by default, so a
secondary jj workspace, git worktree, or fresh clone will not see tasks, artifacts, or
iterations from the primary checkout until it is explicitly attached to the same project.

From the primary checkout, print the current project id:

```sh
gest project
```

Then from each secondary checkout, attach to that same id:

```sh
gest project attach <project-id>
```

Before removing a workspace, detach it so the project row is not left pointing at a stale
path:

```sh
gest project detach
```

Once multiple checkouts share a project, each workspace can call
`gest iteration next --claim --agent <name>` to pull a different unblocked task from the
active phase — this is how phases translate into real parallelism across agents. See
[Dependency Graphs](#dependency-graphs) for how phases and blocking links interact, and
[Orchestrate multiple agents](agents.md#orchestrate-multiple-agents) for the end-to-end
loop. For the full `gest project` reference, see [`gest project`](../cli/project.md).

## Linking

Links create typed relationships between entities. When you link two tasks, gest automatically
creates the reciprocal link on the target.

### Relationship Types

| Type         | Inverse      | Meaning                                      |
|--------------|--------------|----------------------------------------------|
| `blocks`     | `blocked-by` | Source must complete before target can start |
| `blocked-by` | `blocks`     | Source waits on target                       |
| `parent-of`  | `child-of`   | Source is the parent of target               |
| `child-of`   | `parent-of`  | Source is a child of target                  |
| `relates-to` | `relates-to` | General association (symmetric)              |

Link a task to another task:

```sh
gest task link <source-id> blocks <target-id>
```

Link a task to an artifact (no reciprocal link is created):

```sh
gest task link <source-id> relates-to <artifact-id> --artifact
```

## Tagging and Metadata

### Tags

Tags are freeform string labels. Add them at creation time with `--tag` or after the fact:

```sh
gest task tag <id> "api,security"
gest task untag <id> "security"
```

Use tags to filter listings:

```sh
gest task list --tag api
gest artifact list --tag design
```

### Metadata

Metadata stores arbitrary key-value pairs on any entity. Set values with the `-m` flag at
creation time or through the `meta` subcommand:

```sh
gest task meta set <id> complexity high
gest task meta get <id> complexity
```

All metadata is stored as JSON in the database. When local sync is enabled, task and iteration
metadata is mirrored as YAML in the entity's `.gest/` YAML file, and artifact metadata is
mirrored as YAML frontmatter in the entity's `.gest/` Markdown file.

## Storage modes

Gest supports two storage modes:

- **Global store** (default): Entity data lives in a single SQLite database at
  `~/.local/share/gest/gest.db` (Linux) or `~/Library/Application Support/gest/gest.db` (macOS).
  One database, shared across every project on the machine — projects are rows inside it.
- **Local sync**: Same database, plus a `.gest/` directory inside your project. Every mutation
  writes to the database first, then the sync layer exports the affected rows to YAML and
  Markdown files in `.gest/`, grouped into singular per-entity subdirectories (`task/`,
  `artifact/`, `iteration/`, etc.). On read commands the sync layer imports any files that
  are newer than their database rows. This gives you an inspectable, git-commitable mirror
  without giving up ACID guarantees, relational integrity, or efficient queries.

Initialize with `gest init` for global-only or `gest init --local` to also create the
`.gest/` mirror. Remote sync via libsql is opt-in through the `[database]` config section —
see [Configuration](../configuration/index.md) for details.

### Migrating from v0.4.x

If you are coming from gest v0.4.x where data lived in per-entity-type directories under
`.gest/`, run `gest migrate --from v0.4` to import your existing `.gest/` tree into the new
SQLite database. See the [v0.4 → v0.5 migration guide](/docs/0.5/migration/v0-4-to-v0-5) for a
step-by-step walkthrough, or [gest migrate](../cli/migrate.md) for the short CLI reference.
