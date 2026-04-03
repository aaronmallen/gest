# Core Concepts

Gest manages three primary entity types -- tasks, artifacts, and iterations -- and connects them
through links, tags, and metadata. All data is stored as plain files that you can inspect and
version-control alongside your code.

## Tasks

A **task** represents a unit of work. Tasks are stored as TOML files and have the following
properties:

| Field         | Description                                   |
|---------------|-----------------------------------------------|
| `title`       | Short summary of the work                     |
| `description` | Longer explanation (Markdown)                 |
| `status`      | Current state of the task                     |
| `priority`    | Urgency level, 0 (highest) through 4 (lowest) |
| `phase`       | Numeric execution phase for parallel grouping |
| `assigned_to` | Actor responsible for the task                |
| `tags`        | Freeform labels for filtering and grouping    |
| `metadata`    | Arbitrary key-value pairs (TOML table)        |
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
phase numbers execute first.

## Artifacts

An **artifact** is a document -- a spec, ADR, RFC, design note, or any other prose output
generated during development. Artifacts are stored as Markdown files with YAML frontmatter.

| Field         | Description                                                |
|---------------|------------------------------------------------------------|
| `title`       | Document title (extracted from `# heading` if not set)     |
| `body`        | Markdown content                                           |
| `type`        | Document kind (freeform string, e.g. `spec`, `adr`, `rfc`) |
| `tags`        | Freeform labels for filtering                              |
| `metadata`    | Arbitrary key-value pairs (YAML mapping)                   |
| `archived_at` | Timestamp set when the artifact is archived                |

### Artifact Types

The `type` field is a freeform string. Common conventions include:

| Type   | Description                      |
|--------|----------------------------------|
| `spec` | Product or feature specification |
| `adr`  | Architecture Decision Record     |
| `rfc`  | Request for Comments             |
| `note` | General-purpose document         |

You are free to use any value that fits your workflow.

### Archiving

Artifacts can be archived with `gest artifact archive <id>`. Archived artifacts are hidden from
listings and searches by default, but remain on disk. Use `--all` to include them in queries.

## Iterations

An **iteration** groups related tasks into an execution plan. Iterations are stored as TOML
files and track which tasks belong to them, along with their phase assignments.

| Field         | Description                                    |
|---------------|------------------------------------------------|
| `title`       | Name of the iteration                          |
| `description` | Goal or scope of the iteration (Markdown)      |
| `status`      | Current state of the iteration                 |
| `tasks`       | List of task IDs that belong to this iteration |
| `tags`        | Freeform labels                                |
| `metadata`    | Arbitrary key-value pairs (TOML table)         |
| `links`       | Relationships to other entities                |

### Iteration Statuses

| Status      | Meaning                                     |
|-------------|---------------------------------------------|
| `active`    | Currently in progress (default)             |
| `cancelled` | Iteration was deliberately abandoned        |
| `completed` | All tasks finished successfully             |
| `failed`    | *(deprecated)* Alias for `cancelled`        |

### Dependency Graphs

Use `gest iteration graph <id>` to visualize the phased execution plan. The graph shows tasks
grouped by phase, with dependency edges derived from `blocked-by` / `blocks` links between
tasks. This makes it clear which tasks can run in parallel and which must wait.

### Managing Tasks in an Iteration

```sh
gest iteration add <iteration-id> <task-id>      # add a task
gest iteration remove <iteration-id> <task-id>   # remove a task
```

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

Task and iteration metadata is stored as TOML. Artifact metadata is stored as YAML in the
frontmatter.

## Global vs Local Stores

Gest supports two storage modes:

- **Global store** (default): Data lives in your system data directory
  (`~/.local/share/gest/` on Linux, `~/Library/Application Support/gest/` on macOS). This is
  shared across all projects on the machine.
- **Local store**: Data lives in a `.gest/` directory inside your project. This is useful when
  you want to version-control gest data alongside your code or keep tasks scoped to a single
  repository.

Initialize with `gest init` for global or `gest init --local` for local. When a `.gest/`
directory exists in the current project, gest uses it automatically; otherwise it falls back to
the global store.

### File Layout

Tasks are stored as individual TOML files and artifacts as individual Markdown files with YAML
frontmatter. This makes the data inspectable with standard tools, diffable in code review, and
portable between machines.
